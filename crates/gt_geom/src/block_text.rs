//! Block letters from a 5x7 pixel font, each glyph tiled by as few boxes as its lit cells allow, for signs, floor
//! lettering and labels built from real brushes.

use gt_core::{Aabb, DVec3};

pub const COLS: usize = 5;
pub const ROWS: usize = 7;
const LINE_GAP: usize = 2;

fn glyph(c: char) -> Option<[&'static str; ROWS]> {
    Some(match c.to_ascii_uppercase() {
        'A' => [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'B' => ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
        'C' => [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."],
        'D' => ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
        'E' => ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
        'F' => ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
        'G' => [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".###."],
        'H' => ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'I' => [".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
        'J' => ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
        'K' => ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
        'L' => ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
        'M' => ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"],
        'N' => ["#...#", "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#"],
        'O' => [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'P' => ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
        'Q' => [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"],
        'R' => ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
        'S' => [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
        'T' => ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
        'U' => ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'V' => ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
        'W' => ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", ".#.#."],
        'X' => ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
        'Y' => ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
        'Z' => ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
        '0' => [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
        '1' => ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
        '2' => [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
        '3' => ["#####", "...#.", "..#..", "...#.", "....#", "#...#", ".###."],
        '4' => ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
        '5' => ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
        '6' => ["..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###."],
        '7' => ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
        '8' => [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
        '9' => [".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##.."],
        '.' => [".....", ".....", ".....", ".....", ".....", ".##..", ".##.."],
        ',' => [".....", ".....", ".....", ".....", ".##..", "..#..", ".#..."],
        '!' => ["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#.."],
        '?' => [".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#.."],
        '\'' => ["..#..", "..#..", ".#...", ".....", ".....", ".....", "....."],
        '-' => [".....", ".....", ".....", "#####", ".....", ".....", "....."],
        '+' => [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."],
        ':' => [".....", ".##..", ".##..", ".....", ".##..", ".##..", "....."],
        '/' => [".....", "....#", "...#.", "..#..", ".#...", "#....", "....."],
        _ => return None,
    })
}

/// Whether `c` has a glyph. Spaces and line breaks are always accepted by [`layout`].
pub fn supported(c: char) -> bool {
    glyph(c).is_some()
}

/// Runs of lit cells along each line of `lit`, merged across following lines while they repeat exactly, as
/// `[start, line, length, lines]`.
fn merge_runs(lit: &[Vec<bool>]) -> Vec<[usize; 4]> {
    let mut done = Vec::new();
    let mut open: Vec<[usize; 4]> = Vec::new();
    for (l, line) in lit.iter().enumerate() {
        let mut runs = Vec::new();
        let mut i = 0;
        while i < line.len() {
            let start = i;
            while i < line.len() && line[i] {
                i += 1;
            }

            if i > start {
                runs.push((start, i - start));
            }

            i += 1;
        }

        let (kept, closed): (Vec<_>, Vec<_>) = open.into_iter().partition(|o| runs.contains(&(o[0], o[2])));
        done.extend(closed);
        open = runs
            .into_iter()
            .map(|(s, n)| match kept.iter().find(|o| o[0] == s && o[2] == n) {
                Some(o) => [s, o[1], n, o[3] + 1],
                None => [s, l, n, 1],
            })
            .collect();
    }

    done.extend(open);
    done
}

/// Lit cells of a glyph as `[col, row, width, height]` rectangles, row 0 at the top. Runs are merged along rows or
/// along columns, whichever needs fewer boxes, so H is two posts and a bar.
pub fn glyph_rects(c: char) -> Option<Vec<[usize; 4]>> {
    let rows = glyph(c)?;
    let by_row: Vec<Vec<bool>> = rows.iter().map(|r| r.bytes().map(|b| b == b'#').collect()).collect();
    let by_col: Vec<Vec<bool>> = (0..COLS).map(|c| (0..ROWS).map(|r| by_row[r][c]).collect()).collect();
    let across = merge_runs(&by_row);
    let down: Vec<[usize; 4]> = merge_runs(&by_col).into_iter().map(|[s, l, n, m]| [l, s, m, n]).collect();
    let mut best = if down.len() < across.len() { down } else { across };
    best.sort_by_key(|r| (r[1], r[0]));
    Some(best)
}

/// Bounds with `min` as their minimum corner that [`layout`] fills with letters `letter_height` tall and `depth` thick.
pub fn sized_bounds(text: &str, min: DVec3, letter_height: f64, depth: f64, standing: bool, spacing: f64) -> Result<Aabb, String> {
    if letter_height <= 0.0 || depth <= 0.0 {
        return Err("letter height and depth must be positive".into());
    }

    let lines: Vec<usize> = text.lines().map(|l| l.trim_end().chars().count()).collect();
    let spacing = spacing.max(0.0);
    let cols = lines.iter().map(|n| if *n == 0 { 0.0 } else { *n as f64 * (COLS as f64 + spacing) - spacing }).fold(0.0, f64::max);
    if cols <= 0.0 {
        return Err("text has no letters".into());
    }

    let cell = letter_height / ROWS as f64;
    let rows = (lines.len() * ROWS + (lines.len() - 1) * LINE_GAP) as f64 * cell;
    let size = if standing { DVec3::new(cols * cell, rows, depth) } else { DVec3::new(cols * cell, depth, rows) };
    Ok(Aabb::new(min, min + size))
}

/// Lays `text` out inside `bounds` as boxes, one list per visible character. Letters read along +X with square cells
/// as large as fit, centered. Lying letters (the default) put the top of each glyph towards -Z and fill the bounds'
/// height, so they read from above or from a viewer on the +Z side. Standing letters put the top up and fill the
/// bounds' depth, reading from +Z. `spacing` is the gap between letters in cells, `\n` starts a new line.
pub fn layout(text: &str, bounds: &Aabb, standing: bool, spacing: f64) -> Result<Vec<Vec<Aabb>>, String> {
    if let Some(c) = text.chars().find(|c| !(*c == ' ' || *c == '\n' || supported(*c))) {
        return Err(format!("no block letter for {c:?}, use A-Z, 0-9, space and . , ! ? ' - + : /"));
    }

    let lines: Vec<Vec<char>> = text.lines().map(|l| l.trim_end().chars().collect()).collect();
    let spacing = spacing.max(0.0);
    let advance = COLS as f64 + spacing;
    let line_width = |l: &Vec<char>| if l.is_empty() { 0.0 } else { l.len() as f64 * advance - spacing };
    let cols = lines.iter().map(line_width).fold(0.0, f64::max);
    if cols <= 0.0 {
        return Err("text has no letters".into());
    }

    let rows = (lines.len() * ROWS + (lines.len() - 1) * LINE_GAP) as f64;
    let size = bounds.size();
    let (across, down) = if standing { (size.x, size.y) } else { (size.x, size.z) };
    let cell = (across / cols).min(down / rows);
    if cell <= 0.0 || (if standing { size.z } else { size.y }) <= 0.0 {
        return Err("text needs bounds with width, height and depth".into());
    }

    let left = bounds.min.x + (across - cols * cell) * 0.5;
    let top_gap = (down - rows * cell) * 0.5;
    let mut letters = Vec::new();
    for (li, line) in lines.iter().enumerate() {
        let line_left = left + (cols - line_width(line)) * cell * 0.5;
        let line_top = top_gap + (li * (ROWS + LINE_GAP)) as f64 * cell;
        for (ci, c) in line.iter().enumerate() {
            let Some(rects) = glyph_rects(*c) else { continue };
            let x0 = line_left + ci as f64 * advance * cell;
            let boxes = rects
                .iter()
                .map(|&[col, row, w, h]| {
                    let (xa, xb) = (x0 + col as f64 * cell, x0 + (col + w) as f64 * cell);
                    let (da, db) = (line_top + row as f64 * cell, line_top + (row + h) as f64 * cell);
                    if standing {
                        Aabb::new(DVec3::new(xa, bounds.max.y - db, bounds.min.z), DVec3::new(xb, bounds.max.y - da, bounds.max.z))
                    } else {
                        Aabb::new(DVec3::new(xa, bounds.min.y, bounds.min.z + da), DVec3::new(xb, bounds.max.y, bounds.min.z + db))
                    }
                })
                .collect();
            letters.push(boxes);
        }
    }

    Ok(letters)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(c: char) -> Vec<(usize, usize)> {
        let rows = glyph(c).unwrap();
        (0..ROWS).flat_map(|r| (0..COLS).map(move |col| (col, r))).filter(|&(col, r)| rows[r].as_bytes()[col] == b'#').collect()
    }

    #[test]
    fn sized_bounds_give_letters_of_the_asked_height() {
        for standing in [false, true] {
            let bounds = sized_bounds("HI\nOK", DVec3::new(10.0, 0.0, 0.0), 70.0, 8.0, standing, 1.0).unwrap();
            assert_eq!(bounds.min, DVec3::new(10.0, 0.0, 0.0));
            let letters = layout("HI\nOK", &bounds, standing, 1.0).unwrap();
            let h = letters[0].iter().fold(Aabb::EMPTY, |acc, b| acc.union(b)).size();
            let (tall, thick) = if standing { (h.y, h.z) } else { (h.z, h.y) };
            assert!((tall - 70.0).abs() < 1e-9 && (thick - 8.0).abs() < 1e-9, "standing {standing}: {h}");
            assert!((bounds.size().x - 110.0).abs() < 1e-9, "two letters of five cells and one gap, ten units a cell");
        }

        assert!(sized_bounds("A", DVec3::ZERO, 0.0, 8.0, false, 1.0).is_err());
        assert!(sized_bounds(" ", DVec3::ZERO, 70.0, 8.0, false, 1.0).is_err());
    }

    #[test]
    fn every_glyph_is_five_by_seven() {
        for c in ('A'..='Z').chain('0'..='9').chain(".,!?'-+:/".chars()) {
            let rows = glyph(c).unwrap_or_else(|| panic!("{c} missing"));
            assert!(rows.iter().all(|r| r.len() == COLS && r.bytes().all(|b| b == b'#' || b == b'.')), "{c}");
        }
    }

    #[test]
    fn rects_cover_each_lit_cell_exactly_once() {
        for c in ('A'..='Z').chain('0'..='9') {
            let mut covered = Vec::new();
            for [col, row, w, h] in glyph_rects(c).unwrap() {
                for r in row..row + h {
                    for x in col..col + w {
                        covered.push((x, r));
                    }
                }
            }

            covered.sort();
            let mut lit = cells(c);
            lit.sort();
            assert_eq!(covered, lit, "{c}");
        }
    }

    #[test]
    fn rects_merge_runs_into_few_boxes() {
        assert_eq!(glyph_rects('L').unwrap(), vec![[0, 0, 1, 6], [0, 6, 5, 1]]);
        assert_eq!(glyph_rects('T').unwrap().len(), 2);
        assert_eq!(glyph_rects('H').unwrap(), vec![[0, 0, 1, 7], [4, 0, 1, 7], [1, 3, 3, 1]]);
    }

    #[test]
    fn lying_text_is_centered_with_square_cells_and_reads_from_plus_z() {
        let bounds = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(300.0, 4.0, 140.0));
        let letters = layout("HI", &bounds, false, 1.0).unwrap();
        assert_eq!(letters.len(), 2);
        // 11 columns and 7 rows: the depth limits the cell to 20 units, 220 of the 300 wide bounds.
        let all = letters.iter().flatten().fold(Aabb::EMPTY, |acc, b| acc.union(b));
        // The last column of I is empty, so the lit cells end one cell before the layout does.
        assert!((all.min.x - 40.0).abs() < 1e-9 && (all.max.x - 240.0).abs() < 1e-9, "{all:?}");
        assert!(all.min.z.abs() < 1e-9 && (all.max.z - 140.0).abs() < 1e-9, "{all:?}");
        assert!(all.min.y == 0.0 && all.max.y == 4.0);
        // The crossbar of H is row 3 between the posts, the top serif of I sits at the -Z edge.
        assert!(letters[0].iter().any(|b| (b.min.z - 60.0).abs() < 1e-9 && (b.size().x - 60.0).abs() < 1e-9), "{:?}", letters[0]);
        assert!(letters[1].iter().any(|b| b.min.z.abs() < 1e-9 && b.min.x > 160.0));
    }

    #[test]
    fn standing_text_puts_the_top_row_up_and_spans_the_depth() {
        let bounds = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(50.0, 70.0, 8.0));
        let letters = layout("L", &bounds, true, 1.0).unwrap();
        let foot = letters[0].iter().find(|b| b.size().x > 40.0).unwrap();
        assert!(foot.min.y.abs() < 1e-9 && (foot.max.y - 10.0).abs() < 1e-9 && foot.min.z == 0.0 && foot.max.z == 8.0);
    }

    #[test]
    fn lines_spaces_and_unknown_characters() {
        let bounds = Aabb::new(DVec3::ZERO, DVec3::new(1000.0, 8.0, 1000.0));
        assert_eq!(layout("COME\nHOME", &bounds, false, 1.0).unwrap().len(), 8);
        assert_eq!(layout("a b", &bounds, false, 1.0).unwrap().len(), 2);
        assert!(layout("snow ☃", &bounds, false, 1.0).unwrap_err().contains('☃'));
        assert!(layout("   ", &bounds, false, 1.0).is_err());
    }
}
