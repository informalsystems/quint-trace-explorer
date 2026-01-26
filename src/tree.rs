use std::collections::HashSet;

use ratatui::style::Color;

use crate::diff::{DiffKind, DiffResult};

// Display thresholds as percentages of available width
const INLINE_PERCENT: usize = 80;   // Use 80% of available width for inline content
const KEY_PERCENT: usize = 35;      // Map keys get 35% of available width
const VALUE_PERCENT: usize = 45;    // Map values get 45% of available width
const PREVIEW_PERCENT: usize = 70;  // Record previews get 70% of available width

// Indent size in characters
const INDENT_SIZE: usize = 2;

/// Calculate display thresholds based on terminal width and depth
#[derive(Clone, Copy)]
struct DisplayThresholds {
    inline: usize,
    key: usize,
    value: usize,
    preview: usize,
}

impl DisplayThresholds {
    fn new(terminal_width: usize, depth: usize) -> Self {
        let indent_used = depth * INDENT_SIZE;
        let available = terminal_width.saturating_sub(indent_used).max(20);

        Self {
            inline: available * INLINE_PERCENT / 100,
            key: available * KEY_PERCENT / 100,
            value: available * VALUE_PERCENT / 100,
            preview: available * PREVIEW_PERCENT / 100,
        }
    }
}

/// Path to a node in the tree (e.g., ["system", "v1", "state"])
pub type NodePath = Vec<String>;

/// Tracks which nodes are expanded
pub struct ExpansionState {
    expanded: HashSet<NodePath>,
    manual_overrides: HashSet<NodePath>, // Paths explicitly toggled by user
}

impl ExpansionState {
    pub fn new() -> Self {
        Self {
            expanded: HashSet::new(),
            manual_overrides: HashSet::new(),
        }
    }

    pub fn is_expanded(&self, path: &NodePath) -> bool {
        self.expanded.contains(path)
    }

    /// Toggle expansion manually (user action)
    pub fn toggle(&mut self, path: &NodePath) {
        if self.expanded.contains(path) {
            self.expanded.remove(path);
        } else {
            self.expanded.insert(path.clone());
        }
        // Mark as manual override
        self.manual_overrides.insert(path.clone());
    }

    /// Check if a path was manually overridden by user
    pub fn is_manual(&self, path: &NodePath) -> bool {
        self.manual_overrides.contains(path)
    }

    /// Automatically expand a path (not a user action)
    fn auto_expand(&mut self, path: &NodePath) {
        if !self.manual_overrides.contains(path) {
            self.expanded.insert(path.clone());
        }
    }

    /// Expand all ancestor paths leading to changed nodes
    /// This expands the tree to reveal all changes
    pub fn expand_to_changes(&mut self, changed_paths: &[NodePath]) {
        for path in changed_paths {
            // Expand all ancestor paths (not the leaf itself unless it has children)
            for i in 1..path.len() {
                let ancestor = path[0..i].to_vec();
                self.auto_expand(&ancestor);
            }
            // Also expand the path itself (in case it has nested changes)
            self.auto_expand(path);
        }
    }

    /// Clear all expansions and manual overrides
    pub fn clear(&mut self) {
        self.expanded.clear();
        self.manual_overrides.clear();
    }

    /// Expand all given paths (used for expand-all command)
    pub fn expand_all(&mut self, paths: &[NodePath]) {
        for path in paths {
            self.expanded.insert(path.clone());
            self.manual_overrides.insert(path.clone());
        }
    }

    /// Save current expansion state (for backtracking)
    pub fn snapshot(&self) -> HashSet<NodePath> {
        self.expanded.clone()
    }

    /// Restore expansion state from snapshot
    pub fn restore(&mut self, snapshot: &HashSet<NodePath>) {
        // Only restore non-manual items
        let mut new_expanded = HashSet::new();

        // Keep manual overrides
        for path in &self.expanded {
            if self.manual_overrides.contains(path) {
                new_expanded.insert(path.clone());
            }
        }

        // Restore non-manual items from snapshot
        for path in snapshot {
            if !self.manual_overrides.contains(path) {
                new_expanded.insert(path.clone());
            }
        }

        self.expanded = new_expanded;
    }

    /// Expand all items at a given depth, prioritizing changed items
    /// Returns true if any changes were made
    pub fn expand_level(
        &mut self,
        all_expandable_paths: &[NodePath],
        changed_paths: &[NodePath],
        target_depth: usize,
    ) -> bool {
        let changed_set: HashSet<_> = changed_paths.iter().collect();
        let mut changed = false;

        // First, expand changed items at this depth
        for path in all_expandable_paths {
            if path.len() == target_depth
                && changed_set.contains(path)
                && !self.is_expanded(path)
                && !self.is_manual(path)
            {
                self.auto_expand(path);
                changed = true;
            }
        }

        // Then, expand unchanged items at this depth
        for path in all_expandable_paths {
            if path.len() == target_depth
                && !changed_set.contains(path)
                && !self.is_expanded(path)
                && !self.is_manual(path)
            {
                self.auto_expand(path);
                changed = true;
            }
        }

        changed
    }
}

/// A single line in the rendered tree
#[derive(Clone)]
pub struct TreeLine {
    pub path: NodePath,
    pub expandable: bool,
    pub diff: DiffKind,
    pub spans: Vec<StyledSpan>,
}

/// A styled span for syntax highlighting
#[derive(Clone)]
pub struct StyledSpan {
    pub text: String,
    pub style: SpanStyle,
}

/// Style types for syntax highlighting
#[derive(Clone, Copy)]
pub enum SpanStyle {
    Default,
    #[allow(dead_code)]
    String,
    #[allow(dead_code)]
    Number,
    #[allow(dead_code)]
    Boolean,
}

impl SpanStyle {
    pub fn to_color(&self) -> Option<Color> {
        match self {
            SpanStyle::Default => None,
            SpanStyle::String => Some(Color::Cyan),
            SpanStyle::Number => Some(Color::Magenta),
            SpanStyle::Boolean => Some(Color::Blue),
        }
    }
}

impl StyledSpan {
    fn new(text: impl Into<String>, style: SpanStyle) -> Self {
        Self { text: text.into(), style }
    }

    fn default(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Default)
    }

    #[allow(dead_code)]
    fn string(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::String)
    }

    #[allow(dead_code)]
    fn number(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Number)
    }

    #[allow(dead_code)]
    fn boolean(text: impl Into<String>) -> Self {
        Self::new(text, SpanStyle::Boolean)
    }
}

impl TreeLine {
    /// Create a TreeLine with default (unstyled) spans from text
    fn with_default_spans(path: NodePath, text: String, expandable: bool, diff: DiffKind) -> Self {
        let spans = vec![StyledSpan::default(text)];
        Self { path, expandable, diff, spans }
    }
}

/// Get diff marker for a DiffKind
fn diff_marker(diff: DiffKind) -> &'static str {
    match diff {
        DiffKind::Added => "+ ",
        _ => "",
    }
}

/// Format name prefix: "name: " or "" if name is empty
fn name_prefix(name: &str, diff: DiffKind) -> String {
    let marker = diff_marker(diff);
    if name.is_empty() {
        marker.to_string()
    } else {
        format!("{}{}: ", marker, name)
    }
}

/// Format name prefix with icon: "icon name: " or "icon " if name is empty
fn name_prefix_with_icon(icon: &str, name: &str, diff: DiffKind) -> String {
    let marker = diff_marker(diff);
    if name.is_empty() {
        format!("{}{} ", marker, icon)
    } else {
        format!("{}{} {}: ", marker, icon, name)
    }
}

/// Render an itf::Value into tree lines
pub fn render_value(
    name: &str,
    value: &itf::Value,
    path: NodePath,
    expansion: &ExpansionState,
    diff: &DiffResult,
    depth: usize,
    terminal_width: usize,
    collapse_threshold: usize,
) -> Vec<TreeLine> {
    let indent = "  ".repeat(depth);
    let expanded = expansion.is_expanded(&path);
    let diff_kind = diff.get(&path);
    let prefix = name_prefix(name, diff_kind);
    let thresholds = DisplayThresholds::new(terminal_width, depth);

    match value {
        // Leaf values - not expandable
        itf::Value::Bool(b) => {
            let text = format!("{}{}{}", indent, prefix, b);
            vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
        }
        itf::Value::Number(n) => {
            let text = format!("{}{}{}", indent, prefix, n);
            vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
        }
        itf::Value::String(s) => {
            let text = format!("{}{}\"{}\"", indent, prefix, s);
            vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
        }
        itf::Value::BigInt(n) => {
            let text = format!("{}{}{}", indent, prefix, n);
            vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
        }

        // Record - expandable
        itf::Value::Record(fields) => {
            // Check for sum type pattern: {tag: String, value: X}
            if let Some(variant) = classify_sum_type(fields) {
                match variant {
                    SumTypeVariant::Unit(tag) => {
                        // Just show the tag without parentheses
                        let text = format!("{}{}{}", indent, prefix, tag);
                        vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
                    }
                    SumTypeVariant::WithValue(tag, inner_value) => {
                        // Display as Tag(preview of value)
                        let inner_preview = format_value_full(inner_value, thresholds.preview)
                            .unwrap_or_else(|| format_value_short(inner_value));

                        // Check if inner value can be fully inlined
                        let can_inline = format_value_full(inner_value, thresholds.preview).is_some();

                        if can_inline {
                            let text = format!("{}{}{}({})", indent, prefix, tag, inner_preview);
                            vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
                        } else {
                            let icon = if expanded { "▼" } else { "▶" };
                            let icon_prefix = name_prefix_with_icon(icon, name, diff_kind);
                            let text = format!("{}{}{}({})", indent, icon_prefix, tag, inner_preview);
                            let mut lines = vec![TreeLine::with_default_spans(path.clone(), text, true, diff_kind)];
                            if expanded {
                                // Only show the inner value's contents, skip tag
                                let mut value_path = path.clone();
                                value_path.push("value".to_string());
                                lines.extend(render_value_children(inner_value, value_path, expansion, diff, depth + 1, terminal_width, collapse_threshold));
                            }
                            lines
                        }
                    }
                }
            } else if let Some(inline) = format_value_full(value, thresholds.inline) {
                // Small record, show inline without expand
                let text = format!("{}{}{}", indent, prefix, inline);
                vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
            } else {
                // Large record
                let icon = if expanded { "▼" } else { "▶" };
                let icon_prefix = name_prefix_with_icon(icon, name, diff_kind);
                let text = if expanded {
                    format!("{}{}{{", indent, icon_prefix)
                } else {
                    format!("{}{}{{{} fields}}", indent, icon_prefix, fields.len())
                };
                let mut lines = vec![TreeLine::with_default_spans(path.clone(), text, true, diff_kind)];
                if expanded {
                    for (field_name, field_value) in fields.iter() {
                        let mut child_path = path.clone();
                        child_path.push(field_name.clone());
                        lines.extend(render_value(field_name, field_value, child_path, expansion, diff, depth + 1, terminal_width, collapse_threshold));
                    }
                    // Add closing brace
                    let close_text = format!("{}}}", indent);
                    lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, diff_kind));
                }
                lines
            }
        }

        // Map - expandable
        itf::Value::Map(pairs) => {
            if pairs.is_empty() {
                // Empty map, no expand needed
                let text = format!("{}{}Map()", indent, prefix);
                vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
            } else {
                let icon = if expanded { "▼" } else { "▶" };
                let icon_prefix = name_prefix_with_icon(icon, name, diff_kind);
                let text = if expanded {
                    format!("{}{}Map(", indent, icon_prefix)
                } else {
                    format!("{}{}Map({} entries)", indent, icon_prefix, pairs.len())
                };
                let mut lines = vec![TreeLine::with_default_spans(path.clone(), text, true, diff_kind)];
                if expanded {
                    // Group entries by change status
                    let groups = group_by_change_status(pairs.len(), diff, &path);
                    let pairs_vec: Vec<_> = pairs.iter().collect();

                    // Check if any entries are changed - only use collapsing syntax if there's a mix
                    let has_any_changed = groups.iter().any(|(_, _, is_changed)| *is_changed);

                    for (start, group_count, is_changed) in groups {
                        if has_any_changed && !is_changed && group_count >= collapse_threshold && group_count >= 3 {
                            // Create unique path for this collapsed group
                            let mut group_path = path.clone();
                            group_path.push(format!("__collapsed_{}_{}", start, start + group_count - 1));

                            let group_expanded = expansion.is_expanded(&group_path);

                            if group_expanded {
                                // Show entries individually
                                for i in start..(start + group_count) {
                                    let (key, val) = pairs_vec[i];
                                    let mut entry_path = path.clone();
                                    entry_path.push(format!("{}", i));
                                    lines.extend(render_map_entry(key, val, entry_path, expansion, diff, depth + 1, terminal_width, collapse_threshold));
                                }
                            } else {
                                // Show collapsed summary (expandable)
                                let icon = "▶";
                                let summary_text = format!("{}  {} ... ({} unchanged entries)", indent, icon, group_count);
                                lines.push(TreeLine::with_default_spans(group_path, summary_text, true, DiffKind::Unchanged));
                            }
                        } else {
                            // Show entries individually
                            for i in start..(start + group_count) {
                                let (key, val) = pairs_vec[i];
                                let mut entry_path = path.clone();
                                entry_path.push(format!("{}", i));
                                lines.extend(render_map_entry(key, val, entry_path, expansion, diff, depth + 1, terminal_width, collapse_threshold));
                            }
                        }
                    }

                    // Add closing paren
                    let close_text = format!("{})", indent);
                    lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, diff_kind));
                }
                lines
            }
        }

        // Set - expandable only if content can't be shown inline
        itf::Value::Set(items) => {
            let count = items.iter().count();
            let all_simple = all_simple(items.iter());
            let inline = if all_simple {
                format_collection_inline(items.iter(), "Set(", ")", thresholds.inline)
            } else {
                None
            };

            // If we can show inline, no need for expand/collapse
            if let Some(ref inline_str) = inline {
                let text = format!("{}{}{}", indent, prefix, inline_str);
                vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
            } else {
                // Complex or too long - needs expand/collapse
                let icon = if expanded { "▼" } else { "▶" };
                let icon_prefix = name_prefix_with_icon(icon, name, diff_kind);
                let text = if expanded {
                    format!("{}{}Set(", indent, icon_prefix)
                } else {
                    format!("{}{}Set({} items)", indent, icon_prefix, count)
                };
                let mut lines = vec![TreeLine::with_default_spans(path.clone(), text, true, diff_kind)];

                if expanded {
                    let items_vec: Vec<_> = items.iter().collect();

                    let item_lines = render_items_with_collapsing(
                        count,
                        &path,
                        expansion,
                        diff,
                        collapse_threshold,
                        &indent,
                        |i| {
                            let item = items_vec[i];
                            let mut child_path = path.clone();
                            child_path.push(format!("{}", i));
                            render_value("", item, child_path, expansion, diff, depth + 1, terminal_width, collapse_threshold)
                        },
                        |_start, _end, count| format!("... ({} unchanged)", count),
                    );
                    lines.extend(item_lines);

                    // Add closing paren
                    let close_text = format!("{})", indent);
                    lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, diff_kind));
                }
                lines
            }
        }

        // List - expandable only if content can't be shown inline
        itf::Value::List(items) => {
            let all_simple = all_simple(items.iter());
            let inline = if all_simple {
                format_collection_inline(items.iter(), "[", "]", thresholds.inline)
            } else {
                None
            };

            // If we can show inline, no need for expand/collapse
            if let Some(ref inline_str) = inline {
                let text = format!("{}{}{}", indent, prefix, inline_str);
                vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
            } else {
                // Complex or too long - needs expand/collapse
                let icon = if expanded { "▼" } else { "▶" };
                let icon_prefix = name_prefix_with_icon(icon, name, diff_kind);
                let text = if expanded {
                    format!("{}{}[", indent, icon_prefix)
                } else {
                    format!("{}{}List({} items)", indent, icon_prefix, items.len())
                };
                let mut lines = vec![TreeLine::with_default_spans(path.clone(), text, true, diff_kind)];

                if expanded {
                    let item_lines = render_items_with_collapsing(
                        items.len(),
                        &path,
                        expansion,
                        diff,
                        collapse_threshold,
                        &indent,
                        |i| {
                            let item = &items[i];
                            let mut child_path = path.clone();
                            child_path.push(format!("{}", i));
                            render_value(&format!("[{}]", i), item, child_path, expansion, diff, depth + 1, terminal_width, collapse_threshold)
                        },
                        |start, end, count| format!("... ([{}..{}] {} unchanged)", start, end, count),
                    );
                    lines.extend(item_lines);

                    // Add closing bracket
                    let close_text = format!("{}]", indent);
                    lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, diff_kind));
                }
                lines
            }
        }

        // Tuple - expandable only if content can't be shown inline
        itf::Value::Tuple(items) => {
            let all_simple = all_simple(items.iter());
            let inline = if all_simple {
                format_collection_inline(items.iter(), "(", ")", thresholds.inline)
            } else {
                None
            };

            // If we can show inline, no need for expand/collapse
            if let Some(ref inline_str) = inline {
                let text = format!("{}{}{}", indent, prefix, inline_str);
                vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
            } else {
                // Complex or too long - needs expand/collapse
                let icon = if expanded { "▼" } else { "▶" };
                let icon_prefix = name_prefix_with_icon(icon, name, diff_kind);
                let text = if expanded {
                    format!("{}{}(", indent, icon_prefix)
                } else {
                    format!("{}{}Tuple({} items)", indent, icon_prefix, items.len())
                };
                let mut lines = vec![TreeLine::with_default_spans(path.clone(), text, true, diff_kind)];

                if expanded {
                    let items_vec: Vec<_> = items.iter().collect();

                    let item_lines = render_items_with_collapsing(
                        items.len(),
                        &path,
                        expansion,
                        diff,
                        collapse_threshold,
                        &indent,
                        |i| {
                            let item = items_vec[i];
                            let mut child_path = path.clone();
                            child_path.push(format!("{}", i));
                            render_value(&format!("[{}]", i), item, child_path, expansion, diff, depth + 1, terminal_width, collapse_threshold)
                        },
                        |start, end, count| format!("... ([{}..{}] {} unchanged)", start, end, count),
                    );
                    lines.extend(item_lines);

                    // Add closing paren
                    let close_text = format!("{})", indent);
                    lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, diff_kind));
                }
                lines
            }
        }

        itf::Value::Unserializable(u) => {
            let text = format!("{}{}{:?}", indent, prefix, u);
            vec![TreeLine::with_default_spans(path, text, false, diff_kind)]
        }
    }
}

/// Render just the children of a value (without the header line)
/// Used when expanding map entries where the header is already shown
fn render_value_children(
    value: &itf::Value,
    path: NodePath,
    expansion: &ExpansionState,
    diff: &DiffResult,
    depth: usize,
    terminal_width: usize,
    collapse_threshold: usize,
) -> Vec<TreeLine> {
    match value {
        itf::Value::Record(fields) => {
            let mut lines = Vec::new();
            let indent = "  ".repeat(depth);

            // Add opening delimiter
            let open_text = format!("{}{{", indent);
            lines.push(TreeLine::with_default_spans(path.clone(), open_text, false, DiffKind::Unchanged));

            for (field_name, field_value) in fields.iter() {
                let mut field_path = path.clone();
                field_path.push(field_name.clone());
                lines.extend(render_value(field_name, field_value, field_path, expansion, diff, depth + 1, terminal_width, collapse_threshold));
            }

            // Add closing delimiter
            let close_text = format!("{}}}", indent);
            lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, DiffKind::Unchanged));

            lines
        }
        itf::Value::Set(items) => {
            let items_vec: Vec<_> = items.iter().collect();
            render_collection_children(
                items_vec,
                &path,
                expansion,
                diff,
                depth,
                terminal_width,
                collapse_threshold,
                "Set(",
                ")",
                "",
                false,
            )
        }
        itf::Value::List(items) => {
            let items_vec: Vec<_> = items.iter().collect();
            render_collection_children(
                items_vec,
                &path,
                expansion,
                diff,
                depth,
                terminal_width,
                collapse_threshold,
                "[",
                "]",
                "[{}]",
                true,
            )
        }
        itf::Value::Map(pairs) => {
            let mut lines = Vec::new();
            let indent = "  ".repeat(depth);

            // Add opening delimiter
            let open_text = format!("{}Map(", indent);
            lines.push(TreeLine::with_default_spans(path.clone(), open_text, false, DiffKind::Unchanged));

            // Use helper function for each map entry
            for (i, (k, v)) in pairs.iter().enumerate() {
                let mut entry_path = path.clone();
                entry_path.push(format!("{}", i));
                lines.extend(render_map_entry(k, v, entry_path, expansion, diff, depth, terminal_width, collapse_threshold));
            }

            // Add closing delimiter
            let close_text = format!("{})", indent);
            lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, DiffKind::Unchanged));

            lines
        }
        itf::Value::Tuple(items) => {
            let items_vec: Vec<_> = items.iter().collect();
            render_collection_children(
                items_vec,
                &path,
                expansion,
                diff,
                depth,
                terminal_width,
                collapse_threshold,
                "(",
                ")",
                "[{}]",
                true,
            )
        }
        // Simple values have no children
        _ => Vec::new(),
    }
}

/// Classify sum type variants
#[derive(Debug, Clone, Copy)]
enum SumTypeVariant<'a> {
    /// Unit variant (no value): PreVoteStage
    Unit(&'a str),
    /// Variant with value: Some(42)
    WithValue(&'a str, &'a itf::Value),
}

/// Classify a sum type pattern: {tag: String, value: X}
/// Returns Unit if value is empty tuple/record, WithValue otherwise
fn classify_sum_type(fields: &itf::value::Record) -> Option<SumTypeVariant<'_>> {
    // Must have exactly 2 fields: "tag" and "value"
    if fields.len() != 2 {
        return None;
    }

    let tag_value = fields.get("tag")?;
    let inner_value = fields.get("value")?;

    // tag must be a string
    let tag_str = if let itf::Value::String(s) = tag_value {
        s.as_str()
    } else {
        return None;
    };

    // Check if inner value is unit (empty tuple or empty record)
    let is_unit = matches!(
        inner_value,
        itf::Value::Tuple(items) if items.is_empty()
    ) || matches!(
        inner_value,
        itf::Value::Record(fields) if fields.is_empty()
    );

    if is_unit {
        Some(SumTypeVariant::Unit(tag_str))
    } else {
        Some(SumTypeVariant::WithValue(tag_str, inner_value))
    }
}

/// Format mode for value display
#[derive(Debug, Clone, Copy)]
enum FormatMode {
    /// Short format with ellipsis (always succeeds)
    Short,
    /// Full format checking max length (may fail returning None)
    Full(usize),
}

/// Format a value according to the specified mode
/// Short mode always succeeds with ellipsis for complex values
/// Full mode returns None if value is too complex or exceeds max_len
fn format_value(value: &itf::Value, mode: FormatMode) -> Option<String> {
    let result = match value {
        itf::Value::Bool(b) => b.to_string(),
        itf::Value::Number(n) => n.to_string(),
        itf::Value::String(s) => format!("\"{}\"", s),
        itf::Value::BigInt(n) => n.to_string(),
        itf::Value::Record(fields) => {
            // Check for sum type pattern
            match classify_sum_type(fields) {
                Some(SumTypeVariant::Unit(tag)) => tag.to_string(),
                Some(SumTypeVariant::WithValue(tag, inner_value)) => {
                    match mode {
                        FormatMode::Short => format!("{}(...)", tag),
                        FormatMode::Full(max_len) => {
                            // Try to format inner value
                            if let Some(inner_str) = format_value(inner_value, FormatMode::Full(max_len)) {
                                format!("{}({})", tag, inner_str)
                            } else {
                                return None; // Inner value too complex
                            }
                        }
                    }
                }
                None => {
                    match mode {
                        FormatMode::Short => "{ ... }".to_string(),
                        FormatMode::Full(max_len) => {
                            if fields.is_empty() {
                                "{ }".to_string()
                            } else {
                                let parts: Vec<String> = fields
                                    .iter()
                                    .filter_map(|(k, v)| {
                                        format_value(v, FormatMode::Full(max_len)).map(|fv| format!("{}: {}", k, fv))
                                    })
                                    .collect();
                                if parts.len() != fields.len() {
                                    return None; // Some field couldn't be formatted
                                }
                                format!("{{ {} }}", parts.join(", "))
                            }
                        }
                    }
                }
            }
        }
        itf::Value::Map(_) => {
            match mode {
                FormatMode::Short => "Map(...)".to_string(),
                FormatMode::Full(_) => return None, // Maps are complex
            }
        }
        itf::Value::Set(items) => {
            match mode {
                FormatMode::Short => "Set(...)".to_string(),
                FormatMode::Full(max_len) => {
                    let parts: Vec<String> = items
                        .iter()
                        .filter_map(|v| format_value(v, FormatMode::Full(max_len)))
                        .collect();
                    if parts.len() != items.iter().count() {
                        return None;
                    }
                    if parts.is_empty() {
                        "Set()".to_string()
                    } else {
                        format!("Set({})", parts.join(", "))
                    }
                }
            }
        }
        itf::Value::List(items) => {
            match mode {
                FormatMode::Short => "[...]".to_string(),
                FormatMode::Full(max_len) => {
                    let parts: Vec<String> = items
                        .iter()
                        .filter_map(|v| format_value(v, FormatMode::Full(max_len)))
                        .collect();
                    if parts.len() != items.len() {
                        return None;
                    }
                    format!("[{}]", parts.join(", "))
                }
            }
        }
        itf::Value::Tuple(items) => {
            match mode {
                FormatMode::Short => {
                    if items.is_empty() {
                        "()".to_string()
                    } else {
                        "(...)".to_string()
                    }
                }
                FormatMode::Full(max_len) => {
                    let parts: Vec<String> = items
                        .iter()
                        .filter_map(|v| format_value(v, FormatMode::Full(max_len)))
                        .collect();
                    if parts.len() != items.len() {
                        return None;
                    }
                    format!("({})", parts.join(", "))
                }
            }
        }
        itf::Value::Unserializable(_) => "<?>".to_string(),
    };

    // Check length for Full mode
    if let FormatMode::Full(max_len) = mode {
        if result.len() > max_len {
            return None;
        }
    }

    Some(result)
}

/// Short format for map keys (wrapper for backward compatibility)
fn format_value_short(value: &itf::Value) -> String {
    format_value(value, FormatMode::Short).unwrap()
}

/// Format a value fully - returns None if too complex/long (wrapper for backward compatibility)
fn format_value_full(value: &itf::Value, max_len: usize) -> Option<String> {
    format_value(value, FormatMode::Full(max_len))
}

/// Check if a value is "simple" (can be shown inline)
fn is_simple(value: &itf::Value) -> bool {
    matches!(
        value,
        itf::Value::Bool(_)
            | itf::Value::Number(_)
            | itf::Value::String(_)
            | itf::Value::BigInt(_)
    )
}

/// Format a collection inline: {a, b, c} or [1, 2, 3]
fn format_collection_inline<'a>(
    items: impl Iterator<Item = &'a itf::Value>,
    open: &str,
    close: &str,
    max_len: usize,
) -> Option<String> {
    let formatted: Vec<String> = items.map(format_value_short).collect();
    let joined = formatted.join(", ");

    // Only inline if total length is reasonable
    if joined.len() <= max_len {
        Some(format!("{}{}{}", open, joined, close))
    } else {
        None
    }
}

/// Check if all items in a collection are simple
fn all_simple<'a>(items: impl Iterator<Item = &'a itf::Value>) -> bool {
    items.fold(true, |acc, v| acc && is_simple(v))
}

/// Render items with automatic collapsing of unchanged groups
/// Generic function that handles the grouping/collapsing pattern used by all collections
fn render_items_with_collapsing<F, G>(
    total_count: usize,
    path: &NodePath,
    expansion: &ExpansionState,
    diff: &DiffResult,
    collapse_threshold: usize,
    indent: &str,
    render_item: F,
    format_collapsed_summary: G,
) -> Vec<TreeLine>
where
    F: Fn(usize) -> Vec<TreeLine>,
    G: Fn(usize, usize, usize) -> String, // (start, end, count) -> summary text
{
    let mut lines = Vec::new();

    if total_count == 0 {
        return lines;
    }

    // Group items by change status
    let groups = group_by_change_status(total_count, diff, path);

    // Check if any items are changed - only use collapsing syntax if there's a mix
    let has_any_changed = groups.iter().any(|(_, _, is_changed)| *is_changed);

    for (start, group_count, is_changed) in groups {
        if has_any_changed && !is_changed && group_count >= collapse_threshold && group_count >= 3 {
            // Create unique path for this collapsed group
            let mut group_path = path.clone();
            group_path.push(format!("__collapsed_{}_{}", start, start + group_count - 1));

            let group_expanded = expansion.is_expanded(&group_path);

            if group_expanded {
                // Show items individually
                for i in start..(start + group_count) {
                    lines.extend(render_item(i));
                }
            } else {
                // Show collapsed summary (expandable)
                let icon = "▶";
                let summary_text = format!("{}  {} {}", indent, icon, format_collapsed_summary(start, start + group_count - 1, group_count));
                lines.push(TreeLine::with_default_spans(group_path, summary_text, true, DiffKind::Unchanged));
            }
        } else {
            // Show items individually
            for i in start..(start + group_count) {
                lines.extend(render_item(i));
            }
        }
    }

    lines
}

/// Group items by change status, collapsing consecutive unchanged items
/// Returns (start_index, count, is_changed) for each group
fn group_by_change_status(total_count: usize, diff: &DiffResult, base_path: &NodePath) -> Vec<(usize, usize, bool)> {
    if total_count == 0 {
        return vec![];
    }

    let mut groups = Vec::new();
    let mut current_start = 0;
    let mut current_count = 0;
    let mut current_changed = {
        let mut path = base_path.clone();
        path.push("0".to_string());
        let kind = diff.get(&path);
        kind == DiffKind::Added || kind == DiffKind::Removed || kind == DiffKind::Modified
    };

    for i in 0..total_count {
        let mut item_path = base_path.clone();
        item_path.push(format!("{}", i));
        let kind = diff.get(&item_path);
        let is_changed = kind == DiffKind::Added || kind == DiffKind::Removed || kind == DiffKind::Modified;

        if is_changed == current_changed {
            // Continue current group
            current_count += 1;
        } else {
            // Save current group and start new one
            if current_count > 0 {
                groups.push((current_start, current_count, current_changed));
            }
            current_start = i;
            current_count = 1;
            current_changed = is_changed;
        }
    }

    // Save last group
    if current_count > 0 {
        groups.push((current_start, current_count, current_changed));
    }

    groups
}

/// Render a single map entry (key-value pair)
/// Returns TreeLines for the entry and its children (if expanded)
fn render_map_entry(
    key: &itf::Value,
    val: &itf::Value,
    entry_path: NodePath,
    expansion: &ExpansionState,
    diff: &DiffResult,
    depth: usize,
    terminal_width: usize,
    collapse_threshold: usize,
) -> Vec<TreeLine> {
    let mut lines = Vec::new();
    let indent = "  ".repeat(depth);
    let thresholds = DisplayThresholds::new(terminal_width, depth);

    // Format key (try full, fall back to short)
    let key_str = format_value_full(key, thresholds.key)
        .unwrap_or_else(|| format_value_short(key));

    // Try to format value fully inline
    let val_full = format_value_full(val, thresholds.value);
    let can_inline = val_full.is_some();

    // Get diff status for this entry
    let entry_diff = diff.get(&entry_path);
    let marker = diff_marker(entry_diff);

    // Format entry text
    let entry_text = if can_inline {
        // Simple value, no icon needed
        format!("{}  {}{} -> {}", indent, marker, key_str, val_full.unwrap())
    } else {
        // Complex value, show expand icon
        let entry_icon = if expansion.is_expanded(&entry_path) { "▼" } else { "▶" };
        if expansion.is_expanded(&entry_path) {
            // Expanded: don't show preview, children will render delimiters
            format!("{}  {}{} {} ->", indent, marker, entry_icon, key_str)
        } else {
            // Collapsed: show preview
            let val_preview = format_value_short(val);
            format!("{}  {}{} {} -> {}", indent, marker, entry_icon, key_str, val_preview)
        }
    };

    lines.push(TreeLine::with_default_spans(entry_path.clone(), entry_text, !can_inline, entry_diff));

    // If value can't be inlined and this entry is expanded, show children
    if !can_inline && expansion.is_expanded(&entry_path) {
        let child_lines = render_value_children(val, entry_path, expansion, diff, depth + 1, terminal_width, collapse_threshold);
        lines.extend(child_lines);
    }

    lines
}

/// Unified collection rendering for Sets, Lists, and Tuples
/// Handles the common pattern of opening delimiter, collapsing items, closing delimiter
fn render_collection_children(
    items_vec: Vec<&itf::Value>,
    path: &NodePath,
    expansion: &ExpansionState,
    diff: &DiffResult,
    depth: usize,
    terminal_width: usize,
    collapse_threshold: usize,
    open_delimiter: &str,
    close_delimiter: &str,
    item_label_format: &str, // "" for Sets, "[{}]" for Lists/Tuples
    use_range_in_summary: bool, // false for Sets, true for Lists/Tuples
) -> Vec<TreeLine> {
    let mut lines = Vec::new();
    let indent = "  ".repeat(depth);

    // Add opening delimiter
    let open_text = format!("{}{}", indent, open_delimiter);
    lines.push(TreeLine::with_default_spans(path.clone(), open_text, false, DiffKind::Unchanged));

    // Use generic collapsing logic
    let item_count = items_vec.len();
    let item_lines = render_items_with_collapsing(
        item_count,
        path,
        expansion,
        diff,
        collapse_threshold,
        &indent,
        |i| {
            let item = items_vec[i];
            let mut child_path = path.clone();
            child_path.push(format!("{}", i));
            let label = if item_label_format.is_empty() {
                String::new()
            } else {
                item_label_format.replace("{}", &i.to_string())
            };
            render_value(&label, item, child_path, expansion, diff, depth + 1, terminal_width, collapse_threshold)
        },
        |start, end, count| {
            if use_range_in_summary {
                format!("... ([{}..{}] {} unchanged)", start, end, count)
            } else {
                format!("... ({} unchanged)", count)
            }
        },
    );
    lines.extend(item_lines);

    // Add closing delimiter
    let close_text = format!("{}{}", indent, close_delimiter);
    lines.push(TreeLine::with_default_spans(path.clone(), close_text, false, DiffKind::Unchanged));

    lines
}
