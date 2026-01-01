//! Topic tree panel view

use iced::widget::{button, horizontal_rule, horizontal_space, row, scrollable, text, Column};
use iced::{Element, Length};

use crate::mqtt::TopicNode;
use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::types::TreeNodeInfo;
use crate::app::{Message, MqttUi};

/// Static function to collect tree nodes - called from mod.rs for caching
pub fn collect_tree_nodes_static(node: &TopicNode, depth: usize) -> Vec<TreeNodeInfo> {
    let mut result = Vec::new();

    // Sort children alphabetically
    let mut children: Vec<_> = node.children.iter().collect();
    children.sort_by(|a, b| a.0.cmp(b.0));

    for (name, child) in children {
        let info = TreeNodeInfo {
            name: name.clone(),
            full_path: child.full_path.clone(),
            depth,
            has_children: !child.children.is_empty(),
            has_messages: !child.messages.is_empty(),
            message_count: child.message_count,
            is_expanded: child.expanded,
        };
        result.push(info);

        // Recursively add children if expanded
        if child.expanded && !child.children.is_empty() {
            result.extend(collect_tree_nodes_static(child, depth + 1));
        }
    }

    result
}

impl MqttUi {
    pub fn view_topic_tree(&self, id: &str) -> Element<'_, Message> {
        let id_owned = id.to_string();

        let mut content = Column::new().spacing(spacing::XS).padding(spacing::MD);
        content = content.push(
            row![
                text(icons::TOPIC).size(typography::SIZE_MD).color(colors::CYAN),
                text(" Topics")
                    .size(typography::SIZE_LG)
                    .color(colors::CYAN),
                horizontal_space(),
                button(text(icons::TRASH).size(typography::SIZE_SM))
                    .padding(spacing::XS)
                    .style(styles::button_text)
                    .on_press(Message::ClearTopics(id_owned))
            ]
            .spacing(spacing::XS)
            .align_y(iced::Alignment::Center),
        );
        content = content.push(horizontal_rule(1));

        // Use cached nodes if available, otherwise compute
        let has_tree = self.topic_trees.get(id).map(|t| !t.root.children.is_empty()).unwrap_or(false);

        if !has_tree {
            if self.topic_trees.contains_key(id) {
                content = content.push(
                    text("No messages received yet")
                        .size(typography::SIZE_MD)
                        .color(colors::TEXT_MUTED),
                );
            } else {
                content = content.push(
                    text("Waiting for connection...")
                        .size(typography::SIZE_MD)
                        .color(colors::TEXT_MUTED),
                );
            }
        } else {
            let selected = self.selected_topics.get(id).cloned().flatten();

            // Use cached nodes or fall back to computing (for initial render)
            let nodes: Vec<TreeNodeInfo> = self.cached_tree_nodes
                .get(id)
                .cloned()
                .unwrap_or_else(|| {
                    self.topic_trees.get(id)
                        .map(|tree| collect_tree_nodes_static(&tree.root, 0))
                        .unwrap_or_default()
                });

            // Limit visible nodes to prevent UI overload
            const MAX_VISIBLE_NODES: usize = 200;
            for node_info in nodes.into_iter().take(MAX_VISIBLE_NODES) {
                content = content.push(self.render_tree_node(id, &node_info, &selected));
            }
        }

        scrollable(content).height(Length::Fill).into()
    }

    pub fn render_tree_node(
        &self,
        conn_id: &str,
        node: &TreeNodeInfo,
        selected: &Option<String>,
    ) -> Element<'_, Message> {
        let is_selected = selected.as_ref() == Some(&node.full_path);
        let indent = node.depth * 16;

        let chevron = if node.has_children {
            if node.is_expanded {
                icons::CHEVRON_DOWN
            } else {
                icons::CHEVRON_RIGHT
            }
        } else {
            " "
        };

        let name_color = if node.has_messages {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_SECONDARY
        };

        let msg_count = if node.message_count > 0 {
            format!(" ({})", node.message_count)
        } else {
            String::new()
        };

        // Truncate long names with ellipsis
        let max_chars = 30;
        let name = if node.name.chars().count() > max_chars {
            let truncated: String = node.name.chars().take(max_chars - 3).collect();
            format!("{}...", truncated)
        } else {
            node.name.clone()
        };

        let row_content = row![
            text(chevron)
                .size(typography::SIZE_SM)
                .color(colors::TEXT_MUTED),
            text(name)
                .size(typography::SIZE_SM)
                .color(name_color),
            text(msg_count)
                .size(typography::SIZE_XS)
                .color(colors::TEXT_MUTED),
        ]
        .spacing(spacing::XS)
        .align_y(iced::Alignment::Center);

        let full_path = node.full_path.clone();
        let conn_id_str = conn_id.to_string();

        // Logic for nodes with both children and messages:
        // - If collapsed: click expands
        // - If expanded + has messages + not selected: click selects
        // - If expanded + has messages + selected: click collapses
        // - If expanded + no messages: click collapses
        let node_btn = if node.has_children && !node.is_expanded {
            // Has children and collapsed - expand on click
            let path = node.full_path.clone();
            let cid = conn_id.to_string();
            button(row_content)
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_tab(is_selected))
                .on_press(Message::ExpandTopic(cid, path))
        } else if node.has_children && node.is_expanded && node.has_messages && !is_selected {
            // Has children, expanded, has messages, not selected - select on click
            button(row_content)
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_tab(is_selected))
                .on_press(Message::SelectTopic(conn_id_str, full_path))
        } else if node.has_children && node.is_expanded {
            // Has children, expanded, (no messages OR already selected) - collapse on click
            let path = node.full_path.clone();
            let cid = conn_id.to_string();
            button(row_content)
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_tab(is_selected))
                .on_press(Message::CollapseTopic(cid, path))
        } else if node.has_messages {
            // No children, has messages - select topic
            button(row_content)
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_tab(is_selected))
                .on_press(Message::SelectTopic(conn_id_str, full_path))
        } else {
            button(row_content)
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_tab(false))
        };

        row![horizontal_space().width(indent as u16), node_btn,]
            .width(Length::Fill)
            .into()
    }
}
