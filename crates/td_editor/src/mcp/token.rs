//! The secret MCP HTTP requests can be made to carry, so other programs and users on the machine cannot drive the
//! editor.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const ENV: &str = "TRENCHDOT_MCP_TOKEN";

/// The [`super::Saved`] setting behind Require a token in Preferences.
pub const REQUIRE_FILE: &str = "mcp-require-token";

/// Printable ASCII without spaces, so the token goes into an `Authorization` header as it is.
pub fn valid(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|b| b.is_ascii_graphic())
}

/// Where a token given for one run came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Given {
    Flag,
    Env,
}

impl Given {
    pub fn name(self) -> &'static str {
        match self {
            Given::Flag => "--mcp-token",
            Given::Env => ENV,
        }
    }
}

/// Picks the token the HTTP server asks for. A token given for the run wins over the preferences, otherwise the
/// stored one is asked for while the preferences want a token.
pub struct Tokens {
    given: Result<Option<(Given, String)>, String>,
    file: Option<PathBuf>,
    stored: Option<String>,
}

impl Tokens {
    /// `--mcp-token` wins over [`ENV`], and a blank variable counts as unset.
    pub fn new(cli: Option<String>, env: Option<String>, file: Option<PathBuf>) -> Self {
        let given = match (cli, env.as_deref().map(str::trim).filter(|t| !t.is_empty())) {
            (Some(token), _) => Ok(Some((Given::Flag, token))),
            (None, Some(token)) if valid(token) => Ok(Some((Given::Env, token.to_string()))),
            (None, Some(_)) => Err(format!("{ENV} must be printable ASCII without spaces")),
            (None, None) => Ok(None),
        };
        Self { given, file, stored: None }
    }

    pub fn given(&self) -> Option<Given> {
        match &self.given {
            Ok(given) => given.as_ref().map(|(from, _)| *from),
            Err(_) => Some(Given::Env),
        }
    }

    /// The token requests must carry, `None` when they need none. The stored token is read, or made and stored, the
    /// first time it is wanted.
    pub fn wanted(&mut self, require: bool) -> Result<Option<String>, String> {
        match &self.given {
            Err(e) => Err(e.clone()),
            Ok(Some((_, token))) => Ok(Some(token.clone())),
            Ok(None) if !require => Ok(None),
            Ok(None) => {
                if self.stored.is_none() {
                    self.stored = Some(stored(self.file.as_deref())?);
                }

                Ok(self.stored.clone())
            }
        }
    }
}

/// The token kept in `file`, made and kept there on first use.
fn stored(file: Option<&Path>) -> Result<String, String> {
    if let Some(token) = file.and_then(read) {
        return Ok(token);
    }

    let token = generate().ok_or("the system has no secure random numbers to make an MCP token from")?;
    if let Some(path) = file
        && let Err(e) = store(path, &token)
    {
        eprintln!("TrenchDot: could not store the MCP token in {}, it changes every start: {e}", path.display());
    }

    Ok(token)
}

fn generate() -> Option<String> {
    use ring::rand::SecureRandom;
    let mut bytes = [0u8; 32];
    ring::rand::SystemRandom::new().fill(&mut bytes).ok()?;
    Some(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn read(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Readable but closed to group and others means this user owns it, so nobody else planted it in a shared temp
        // folder.
        if file.metadata().ok()?.permissions().mode() & 0o077 != 0 {
            return None;
        }
    }

    #[cfg(windows)]
    if !private::only_mine(&file) {
        return None;
    }

    let mut text = String::new();
    file.take(1024).read_to_string(&mut text).ok()?;
    let token = text.trim();
    valid(token).then(|| token.to_string())
}

fn store(path: &Path, token: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let _ = std::fs::remove_file(path);
    #[cfg(windows)]
    let mut file = private::create(path)?;
    #[cfg(not(windows))]
    let mut file = {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        options.open(path)?
    };
    file.write_all(token.as_bytes())
}

/// Files only this user can open, whatever the folder they are in grants others. A file takes the rights of its folder
/// otherwise, and the temp folder can grant them to other accounts.
#[cfg(windows)]
mod private {
    use std::fs::File;
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use std::path::Path;
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, LocalFree};
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, ACL_REVISION, ACL_SIZE_INFORMATION, AclSizeInformation, AddAccessAllowedAce, DACL_SECURITY_INFORMATION, EqualSid,
        GetAce, GetAclInformation, GetLengthSid, GetTokenInformation, InitializeAcl, InitializeSecurityDescriptor, OWNER_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR, PSID, SE_DACL_PROTECTED, SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR, SetSecurityDescriptorControl, SetSecurityDescriptorDacl,
        SetSecurityDescriptorOwner, TOKEN_QUERY, TOKEN_USER, TokenUser,
    };
    use windows_sys::Win32::Storage::FileSystem::{CREATE_NEW, CreateFileW, FILE_ALL_ACCESS, FILE_ATTRIBUTE_NORMAL};
    use windows_sys::Win32::System::SystemServices::{ACCESS_ALLOWED_ACE_TYPE, SECURITY_DESCRIPTOR_REVISION};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    /// The TOKEN_USER of this process, which holds the SID it runs as. u64s keep it aligned.
    struct User(Vec<u64>);

    impl User {
        fn current() -> io::Result<Self> {
            let mut token: HANDLE = null_mut();
            if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
                return Err(io::Error::last_os_error());
            }

            let mut len = 0;
            unsafe { GetTokenInformation(token, TokenUser, null_mut(), 0, &mut len) };
            let mut info = vec![0u64; (len as usize).div_ceil(8)];
            let got = unsafe { GetTokenInformation(token, TokenUser, info.as_mut_ptr().cast(), len, &mut len) } != 0;
            let error = io::Error::last_os_error();
            unsafe { CloseHandle(token) };
            if got { Ok(Self(info)) } else { Err(error) }
        }

        fn sid(&self) -> PSID {
            unsafe { (*self.0.as_ptr().cast::<TOKEN_USER>()).User.Sid }
        }
    }

    /// A new file owned by this user, with a DACL that lets only this user in and takes nothing from the folder.
    pub fn create(path: &Path) -> io::Result<File> {
        let user = User::current()?;
        let sid = user.sid();
        let len = size_of::<ACL>() + size_of::<ACCESS_ALLOWED_ACE>() - size_of::<u32>() + unsafe { GetLengthSid(sid) } as usize;
        let mut acl_bytes = vec![0u32; len.div_ceil(4)];
        let acl = acl_bytes.as_mut_ptr().cast::<ACL>();
        let mut descriptor = SECURITY_DESCRIPTOR { Revision: 0, Sbz1: 0, Control: 0, Owner: null_mut(), Group: null_mut(), Sacl: null_mut(), Dacl: null_mut() };
        let sd: PSECURITY_DESCRIPTOR = (&raw mut descriptor).cast();
        let made = unsafe {
            InitializeAcl(acl, (acl_bytes.len() * 4) as u32, ACL_REVISION) != 0
                && AddAccessAllowedAce(acl, ACL_REVISION, FILE_ALL_ACCESS, sid) != 0
                && InitializeSecurityDescriptor(sd, SECURITY_DESCRIPTOR_REVISION) != 0
                && SetSecurityDescriptorOwner(sd, sid, 0) != 0
                && SetSecurityDescriptorDacl(sd, 1, acl, 0) != 0
                && SetSecurityDescriptorControl(sd, SE_DACL_PROTECTED, SE_DACL_PROTECTED) != 0
        };
        if !made {
            return Err(io::Error::last_os_error());
        }

        let attributes = SECURITY_ATTRIBUTES { nLength: size_of::<SECURITY_ATTRIBUTES>() as u32, lpSecurityDescriptor: sd, bInheritHandle: 0 };
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let handle = unsafe { CreateFileW(wide.as_ptr(), GENERIC_WRITE, 0, &attributes, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, null_mut()) };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        Ok(unsafe { File::from_raw_handle(handle) })
    }

    /// This user owns the file and nobody else is let in.
    pub fn only_mine(file: &File) -> bool {
        let Ok(user) = User::current() else { return false };
        let (mut owner, mut dacl, mut sd) = (null_mut(), null_mut(), null_mut());
        let wanted = OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION;
        if unsafe { GetSecurityInfo(file.as_raw_handle(), SE_FILE_OBJECT, wanted, &mut owner, null_mut(), &mut dacl, null_mut(), &mut sd) } != 0 {
            return false;
        }

        // A missing DACL lets everybody in.
        let mine = !owner.is_null() && !dacl.is_null() && unsafe { EqualSid(owner, user.sid()) != 0 && only_for(dacl, user.sid()) };
        unsafe { LocalFree(sd) };
        mine
    }

    /// Every entry of `acl` lets `sid` in, and no other.
    unsafe fn only_for(acl: *const ACL, sid: PSID) -> bool {
        let mut size = ACL_SIZE_INFORMATION::default();
        if unsafe { GetAclInformation(acl, (&raw mut size).cast(), size_of::<ACL_SIZE_INFORMATION>() as u32, AclSizeInformation) } == 0 {
            return false;
        }

        (0..size.AceCount).all(|i| {
            let mut ace = null_mut();
            unsafe {
                GetAce(acl, i, &mut ace) != 0
                    && u32::from((*ace.cast::<ACE_HEADER>()).AceType) == ACCESS_ALLOWED_ACE_TYPE
                    && EqualSid((&raw mut (*ace.cast::<ACCESS_ALLOWED_ACE>()).SidStart).cast(), sid) != 0
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_made_token_is_stored_and_reused() {
        let dir = std::env::temp_dir().join(format!("td_mcp_token_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("data").join("mcp-token");
        let mut tokens = Tokens::new(None, None, Some(file.clone()));
        assert_eq!(tokens.wanted(false), Ok(None));
        assert!(!file.exists(), "nothing is made while no token is wanted");
        let first = tokens.wanted(true).unwrap().unwrap();
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), first);
        assert_eq!(tokens.wanted(false), Ok(None));
        assert_eq!(tokens.wanted(true).unwrap().as_deref(), Some(first.as_str()));
        assert_eq!(stored(Some(&file)).unwrap(), first);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
        }

        #[cfg(windows)]
        assert!(private::only_mine(&std::fs::File::open(&file).unwrap()), "only this user can open it");

        std::fs::write(&file, "not a token").unwrap();
        let replaced = stored(Some(&file)).unwrap();
        assert_ne!(replaced, first);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), replaced);
        assert_ne!(stored(None).unwrap(), stored(None).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_given_token_wins_over_the_preferences() {
        let dir = std::env::temp_dir().join(format!("td_mcp_token_given_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = Some(dir.join("mcp-token"));
        let mut flag = Tokens::new(Some("from-cli".into()), Some("from-env".into()), file.clone());
        assert_eq!(flag.given(), Some(Given::Flag));
        assert_eq!(flag.wanted(false).unwrap().as_deref(), Some("from-cli"));
        let mut env = Tokens::new(None, Some(" from-env\n".into()), file.clone());
        assert_eq!(env.given(), Some(Given::Env));
        assert_eq!(env.wanted(true).unwrap().as_deref(), Some("from-env"));
        assert!(!dir.exists());

        let mut bad = Tokens::new(None, Some("two words".into()), file.clone());
        assert!(bad.wanted(false).is_err());
        let mut blank = Tokens::new(None, Some("  ".into()), file);
        assert_eq!(blank.given(), None);
        assert_eq!(blank.wanted(false), Ok(None));
        assert_eq!(blank.wanted(true).unwrap().map(|t| t.len()), Some(64));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tokens_are_printable_ascii_without_spaces() {
        assert!(valid("abc-123_~+/="));
        assert!(!valid(""));
        assert!(!valid("a b"));
        assert!(!valid("tab\there"));
        assert!(!valid("caf\u{e9}"));
    }

    #[test]
    fn a_token_file_others_can_read_is_not_trusted() {
        let dir = std::env::temp_dir().join(format!("td_mcp_token_mode_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("mcp-token");
        // On Windows a plain write takes the temp folder's rights, which let SYSTEM and the administrators in as well.
        std::fs::write(&file, "planted").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        }

        let made = stored(Some(&file)).unwrap();
        assert_ne!(made, "planted");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), made, "the token made instead replaces it");
        #[cfg(windows)]
        {
            assert!(private::only_mine(&std::fs::File::open(&file).unwrap()));
            let hosts = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join(r"System32\drivers\etc\hosts");
            assert!(!private::only_mine(&std::fs::File::open(hosts).unwrap()), "a file SYSTEM owns");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
