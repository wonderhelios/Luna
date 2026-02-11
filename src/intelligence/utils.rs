#![allow(dead_code)]
use core::code_chunk::CodeChunkMiddle;
use std::collections::BTreeSet;
use tree_sitter::Node;

pub const WITH_LINE_NUM: bool = true;

pub static INDENT_STEP: &str = "\t";

type FnWalkTree =
    fn(&Node, &str, &[u8], &BTreeSet<String>, &Vec<EntityPosition>, &mut String) -> bool;
type FnGetEnPosition = fn(node: &Node, code: &[u8], en_positions: &mut Vec<EntityPosition>);

pub fn node_text(node: &Node, code: &[u8]) -> String {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();
    let node_text = &code[start_byte..end_byte];
    let node_str = std::str::from_utf8(node_text).expect("Invalid UTF-8");
    node_str.to_string()
}

pub fn result_append_impl(result: &mut String, new_str: &str) {
    result.push_str(new_str);
}

pub fn result_append_with_line_num(
    result: &mut String,
    new_str: &str,
    line: usize,
    last_line: &mut usize,
) {
    if line == *last_line {
        result.push_str(new_str);
    } else {
        result.push_str(&format!("{:<3} {}", line + 1, new_str));
        *last_line = line;
    }
}

pub fn result_append(result: &mut String, new_str: &str, line: usize, last_line: &mut usize) {
    if WITH_LINE_NUM {
        result_append_with_line_num(result, new_str, line, last_line);
    } else {
        result_append_impl(result, new_str);
    }
}

pub fn is_intersecting(
    code_chunks: &Vec<CodeChunkMiddle>,
    start_line: usize,
    end_line: usize,
) -> bool {
    if code_chunks.len() > 0 {
        let first_element = &code_chunks[0];
        let last_index = code_chunks.len() - 1;
        let last_element = &code_chunks[last_index];
        if start_line <= first_element.start_line
            && end_line >= last_element.end_line
            && end_line - start_line > 100
        {
            return false;
        }
    }
    for chunk in code_chunks {
        if start_line <= chunk.end_line && chunk.start_line <= end_line {
            return true;
        }
    }
    return false;
}

pub fn is_definition_only(
    definition_symbols: &BTreeSet<String>,
    en_poses: &Vec<EntityPosition>,
) -> bool {
    return definition_symbols.len() > 0 || en_poses.len() > 0;
}

pub fn is_definition_hitted(
    definition_symbols: &BTreeSet<String>,
    en_poses: &Vec<EntityPosition>,
    symbol_name: &String,
    start_line: usize,
    end_line: usize,
) -> bool {
    definition_symbols.contains(symbol_name) || match_any(en_poses, start_line, end_line)
}

pub fn get_body(
    node: &Node,
    code: &[u8],
    indent: &str,
    start_len: usize,
    result: &mut String,
) -> bool {
    if WITH_LINE_NUM {
        return get_body_with_line_num(node, code, indent, start_len, result);
    } else {
        return get_body_impl(node, code, indent, start_len, result);
    }
}

pub fn get_body_impl(
    node: &Node,
    code: &[u8],
    indent: &str,
    start_len: usize,
    result: &mut String,
) -> bool {
    result.truncate(start_len);
    result_append_impl(result, &format!("{} {}\n", indent, node_text(node, code))); // class body
    return true;
}

pub fn get_body_with_line_num(
    node: &Node,
    code: &[u8],
    indent: &str,
    start_len: usize,
    result: &mut String,
) -> bool {
    result.truncate(start_len);
    let start_row = node.start_position().row + 1;
    let add_row_body = node_text(node, code)
        .lines()
        .enumerate()
        .map(|(i, l)| format!("{} {}{}", start_row + i, indent, l))
        .collect::<Vec<String>>()
        .join("\n");
    result_append_impl(result, &format!("{}\n", add_row_body));
    return true;
}

pub fn rows_to_chunks<'a>(relative_path: &'a str, rows: &'a [usize]) -> Vec<CodeChunkMiddle<'a>> {
    let mut chunks: Vec<CodeChunkMiddle> = Vec::new();
    for &row in rows {
        chunks.push(CodeChunkMiddle {
            path: relative_path,
            text: relative_path,
            alias: 0,
            start_line: row - 1,
            end_line: row - 1,
        });
    }
    chunks
}

// entity position
#[derive(Debug, Eq, Clone)]
pub struct EntityPosition {
    start_line: usize,
    end_line: usize,
    entity_name: String,
}

impl EntityPosition {
    pub fn new(start_line: usize, end_line: usize, entity_name: &str) -> Self {
        EntityPosition {
            start_line,
            end_line,
            entity_name: entity_name.to_string(),
        }
    }
}
impl PartialEq for EntityPosition {
    fn eq(&self, other: &Self) -> bool {
        self.start_line == other.start_line && self.end_line == other.end_line
    }
}

pub fn intersect(cur_en_pos: &EntityPosition, chunks: &Vec<CodeChunkMiddle>) -> bool {
    for chunk in chunks {
        if chunk.start_line <= cur_en_pos.end_line && cur_en_pos.start_line <= chunk.end_line {
            return true;
        }
    }
    return false;
}
pub fn include(cur_en_pos: &EntityPosition, en_pos: &EntityPosition) -> bool {
    if cur_en_pos.start_line <= en_pos.start_line && cur_en_pos.end_line >= en_pos.end_line {
        return true;
    }
    return false;
}
pub fn no_child_chunk_intersecting(
    cur_en_pos: &EntityPosition,
    en_poses: &Vec<EntityPosition>,
    chunks: &Vec<CodeChunkMiddle>,
) -> bool {
    for en_pos in en_poses {
        if en_pos != cur_en_pos && intersect(en_pos, chunks) && include(cur_en_pos, en_pos) {
            return false;
        }
    }
    return true;
}

pub fn precise_intersected(
    en_poses: &Vec<EntityPosition>,
    code_chunks: &Vec<CodeChunkMiddle>,
) -> Vec<EntityPosition> {
    let mut matched_en_poses = Vec::<EntityPosition>::new();
    for en_pos in en_poses {
        if intersect(&en_pos, code_chunks)
            && no_child_chunk_intersecting(&en_pos, &en_poses, code_chunks)
        {
            matched_en_poses.push(en_pos.clone());
        }
    }
    return matched_en_poses;
}

pub fn match_any(en_poses: &Vec<EntityPosition>, start_line: usize, end_line: usize) -> bool {
    for en_pos in en_poses {
        if start_line == en_pos.start_line && end_line == en_pos.end_line {
            return true;
        }
    }
    return false;
}

// interface for all langguage
pub fn get_definition_impl(
    node: &Node,
    code: &[u8],
    rows: &[usize],
    path: &str,
    walk_tree: FnWalkTree,
    get_entity_position: FnGetEnPosition,
) -> Result<String, tree_sitter::QueryError> {
    let mut result = "".to_string();
    let mut definition_symbols = BTreeSet::<String>::new();
    let code_chunks = rows_to_chunks(path, rows);
    let mut en_positions = Vec::<EntityPosition>::new();
    get_entity_position(node, code, &mut en_positions);
    let precise_en_poses = precise_intersected(&en_positions, &code_chunks);

    walk_tree(
        node,
        "",
        code,
        &mut definition_symbols,
        &precise_en_poses,
        &mut result,
    );
    Ok(result)
}

pub fn gen_filemap_impl(
    node: &Node,
    code: &[u8],
    walk_tree: FnWalkTree,
) -> Result<String, tree_sitter::QueryError> {
    let mut result = "".to_string();

    let mut definition_symbols = BTreeSet::<String>::new();
    let en_positions = Vec::<EntityPosition>::new();

    walk_tree(
        node,
        "",
        code,
        &mut definition_symbols,
        &en_positions,
        &mut result,
    );
    Ok(result)
}

pub fn complete_chunks_impl(
    node: &Node,
    code: &[u8],
    chunks: &Vec<CodeChunkMiddle>,
    walk_tree: FnWalkTree,
    get_entity_position: FnGetEnPosition,
    result: &mut String,
) -> Result<(), tree_sitter::QueryError> {
    let mut definition_symbols = BTreeSet::<String>::new();
    let mut en_positions = Vec::<EntityPosition>::new();
    get_entity_position(node, code, &mut en_positions);
    let precise_en_poses = precise_intersected(&en_positions, &chunks);

    walk_tree(
        node,
        "",
        code,
        &mut definition_symbols,
        &precise_en_poses,
        result,
    );
    Ok(())
}
