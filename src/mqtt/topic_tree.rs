use std::collections::HashMap;

use super::message::MqttMessage;

#[derive(Debug, Clone, Default)]
pub struct TopicNode {
    #[allow(dead_code)]
    pub name: String,
    pub full_path: String,
    pub children: HashMap<String, TopicNode>,
    pub messages: Vec<MqttMessage>,
    pub message_count: usize,
    pub expanded: bool,
}

impl TopicNode {
    pub fn new(name: &str, full_path: &str) -> Self {
        Self {
            name: name.to_string(),
            full_path: full_path.to_string(),
            children: HashMap::new(),
            messages: Vec::new(),
            message_count: 0,
            expanded: false,
        }
    }

    pub fn insert_message(&mut self, topic_parts: &[&str], message: MqttMessage) {
        self.message_count += 1;

        if topic_parts.is_empty() {
            self.messages.push(message);
            // Keep last 100 messages per topic
            if self.messages.len() > 100 {
                self.messages.remove(0);
            }
            return;
        }

        let part = topic_parts[0];
        let remaining = &topic_parts[1..];

        let child_path = if self.full_path.is_empty() {
            part.to_string()
        } else {
            format!("{}/{}", self.full_path, part)
        };

        let child = self
            .children
            .entry(part.to_string())
            .or_insert_with(|| TopicNode::new(part, &child_path));

        child.insert_message(remaining, message);
    }

    pub fn last_message(&self) -> Option<&MqttMessage> {
        self.messages.last()
    }

    #[allow(dead_code)]
    pub fn total_children_count(&self) -> usize {
        let mut count = self.children.len();
        for child in self.children.values() {
            count += child.total_children_count();
        }
        count
    }

    #[allow(dead_code)]
    pub fn sorted_children(&self) -> Vec<(&String, &TopicNode)> {
        let mut children: Vec<_> = self.children.iter().collect();
        children.sort_by(|a, b| a.0.cmp(b.0));
        children
    }
}

#[derive(Debug, Default)]
pub struct TopicTree {
    pub root: TopicNode,
    pub total_messages: usize,
    pub total_topics: usize,
}

impl TopicTree {
    pub fn new() -> Self {
        Self {
            root: TopicNode::new("", ""),
            total_messages: 0,
            total_topics: 0,
        }
    }

    pub fn insert(&mut self, message: MqttMessage) {
        let topic = message.topic.clone();
        let parts: Vec<&str> = topic.split('/').collect();
        self.root.insert_message(&parts, message);
        self.total_messages += 1;
        self.recalculate_topic_count();
    }

    fn recalculate_topic_count(&mut self) {
        self.total_topics = Self::count_topics(&self.root);
    }

    fn count_topics(node: &TopicNode) -> usize {
        let mut count = if node.messages.is_empty() { 0 } else { 1 };
        for child in node.children.values() {
            count += Self::count_topics(child);
        }
        count
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.root = TopicNode::new("", "");
        self.total_messages = 0;
        self.total_topics = 0;
    }

    #[allow(dead_code)]
    pub fn get_node(&self, topic: &str) -> Option<&TopicNode> {
        let parts: Vec<&str> = topic.split('/').collect();
        Self::get_node_recursive(&self.root, &parts)
    }

    fn get_node_recursive<'a>(node: &'a TopicNode, parts: &[&str]) -> Option<&'a TopicNode> {
        if parts.is_empty() {
            return Some(node);
        }

        let part = parts[0];
        if let Some(child) = node.children.get(part) {
            Self::get_node_recursive(child, &parts[1..])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_all_topics(&self) -> Vec<String> {
        let mut topics = Vec::new();
        Self::collect_topics(&self.root, &mut topics);
        topics.sort();
        topics
    }

    #[allow(dead_code)]
    fn collect_topics(node: &TopicNode, topics: &mut Vec<String>) {
        if !node.messages.is_empty() {
            topics.push(node.full_path.clone());
        }
        for child in node.children.values() {
            Self::collect_topics(child, topics);
        }
    }

    pub fn get_latest_message(&self, topic: &str) -> Option<&MqttMessage> {
        self.get_node(topic).and_then(|n| n.last_message())
    }

    pub fn expand(&mut self, topic: &str) {
        if let Some(node) = self.get_node_mut(topic) {
            node.expanded = true;
        }
    }

    pub fn collapse(&mut self, topic: &str) {
        if let Some(node) = self.get_node_mut(topic) {
            node.expanded = false;
        }
    }

    fn get_node_mut(&mut self, topic: &str) -> Option<&mut TopicNode> {
        let parts: Vec<&str> = topic.split('/').collect();
        Self::get_node_recursive_mut(&mut self.root, &parts)
    }

    fn get_node_recursive_mut<'a>(
        node: &'a mut TopicNode,
        parts: &[&str],
    ) -> Option<&'a mut TopicNode> {
        if parts.is_empty() {
            return Some(node);
        }

        let part = parts[0];
        if let Some(child) = node.children.get_mut(part) {
            Self::get_node_recursive_mut(child, &parts[1..])
        } else {
            None
        }
    }
}
