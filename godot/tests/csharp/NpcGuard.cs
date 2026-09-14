using Godot;

// Fixture for the C# entity scanner. The test project is a standard (non .NET) build, so this file is only read as text.
[GlobalClass]
[GodotTrenchEntity("npc_guard", Description = "Patrolling guard", Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level);

    [Export] public float WalkSpeed { get; set; } = 3.5f;
    [Export] public bool StartAsleep = true;
    [Export] public Vector3 PatrolOffset { get; set; } = new Vector3(0, 0, 128);

    [GodotTrenchInput] public void Alert(Node activator) { }
    [GodotTrenchInput] public void GoToSleep() { }

    public void NotAnInput() { }
}

[GlobalClass]
[GodotTrenchEntity("func_vault_door", Solid = true, Description = "Vault door opened by a code")]
public partial class VaultDoor : AnimatableBody3D
{
    [Signal] public delegate void UnlockedEventHandler();

    [Export] public string Code = "1234";

    public void TryCode(string code) { }
}
