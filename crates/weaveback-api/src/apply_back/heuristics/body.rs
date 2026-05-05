// weaveback-api/src/apply_back/heuristics/body.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

/// For a `MacroBodyWithVars` span: reconstruct the body template with only the
/// literal (non-variable) parts updated.
///
/// Algorithm:
///  1. Split `body_line` into alternating literal/variable segments via `%%(...)`.
///  2. Walk `old_expanded` to extract the runtime value of each variable.
///  3. Walk `new_expanded` to extract the new literal parts (variable values held fixed).
///  4. Rebuild body using original variable references and new literals.
pub(in crate::apply_back) fn attempt_macro_body_fix(
    body_line: &str,
    old_expanded: &str,
    new_expanded: &str,
    sigil: char,
) -> Option<String> {
    if old_expanded == new_expanded { return None; }

    // If the body line is exactly the expanded text, just return the new text.
    if body_line.trim() == old_expanded.trim() {
        return Some(new_expanded.to_string());
    }

    let special_esc = regex::escape(&sigil.to_string());
    let var_re = Regex::new(&format!(r"{}[(][A-Za-z_][A-Za-z0-9_]*[)]", special_esc)).ok()?;

    let mut lits: Vec<&str> = Vec::new();
    let mut var_refs: Vec<&str> = Vec::new();
    let mut pos = 0;
    for m in var_re.find_iter(body_line) {
        lits.push(&body_line[pos..m.start()]);
        var_refs.push(m.as_str());
        pos = m.end();
    }
    lits.push(&body_line[pos..]);

    if var_refs.is_empty() {
        // No variables. Just try to replace old_expanded in body_line.
        if let Some(start) = body_line.find(old_expanded) {
            let mut s = body_line.to_string();
            s.replace_range(start..start + old_expanded.len(), new_expanded);
            return Some(s);
        }
        return None;
    }

    let mut var_vals: Vec<&str> = Vec::new();
    let mut rem = old_expanded;
    for i in 0..var_refs.len() {
        rem = rem.strip_prefix(lits[i])?;
        let next_lit = lits[i + 1];
        let end = if next_lit.is_empty() && i + 1 == var_refs.len() {
            rem.len()
        } else if next_lit.is_empty() {
            return None; // adjacent variables — ambiguous
        } else {
            rem.find(next_lit)?
        };
        var_vals.push(&rem[..end]);
        rem = &rem[end..];
    }
    if !rem.starts_with(lits[var_refs.len()]) { return None; }

    let mut new_lits: Vec<String> = Vec::new();
    let mut new_rem = new_expanded;
    for var_val in &var_vals {
        let var_pos = new_rem.find(var_val)?;
        new_lits.push(new_rem[..var_pos].to_string());
        new_rem = &new_rem[var_pos + var_val.len()..];
    }
    new_lits.push(new_rem.to_string());

    let mut new_body = String::new();
    for (i, var_ref) in var_refs.iter().enumerate() {
        new_body.push_str(&new_lits[i]);
        new_body.push_str(var_ref);
    }
    new_body.push_str(&new_lits[var_refs.len()]);

    if new_body == body_line { None } else { Some(new_body) }
}

