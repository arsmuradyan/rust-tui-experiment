struct SSHGroup<'a> {
    path: &'a str,
    hosts: Vec<&'a str>
}
struct SSHStore<'a> {
    nodes: Vec<UITreeNode<'a>>,
    groups: Vec<SSHGroup<'a>>
}
#[derive(Clone, Copy)]
struct UITreeNode<'a> {
    path: &'a str,
    name: &'a str,
    is_dir: bool
}
impl <'a> SSHStore<'a> {
    fn groups_to_nodes(&mut self) {
        for group in &self.groups {
            let nodes: Vec<UITreeNode> =  self.nodes.iter().filter(|node| node.name == group.path).cloned().collect();
            if nodes.len() > 0 {
                self.nodes.extend(group.to_ui_tree_node(Some(nodes[0])))
            } else {
                self.nodes.extend(group.to_ui_tree_node(None))
            }
        }
    }
}
impl<'a> SSHGroup<'a> {
    fn to_ui_tree_node(&self, folder_node: Option<UITreeNode<'a>>) -> Vec<UITreeNode<'a>> {
        let mut nodes: Vec<UITreeNode<'a>> = vec![];
        match folder_node {
            None => {
                let path_node = UITreeNode{
                    path: self.path,
                    name: self.path,
                    is_dir: true
                };
                nodes.push(path_node);
            }
            Some(_) => {
            }
        }
        for host in &self.hosts {
            let node = UITreeNode{
                path: self.path,
                name: &host,
                is_dir: false
            };
            nodes.push(node)
        }

        return nodes
    }
}
fn main() {
    let ssh_groups = vec![SSHGroup{
        path: "hello",
        hosts: vec!["server1.com", "server2.com"]
    }, SSHGroup{
          path: "hello",
          hosts: vec!["server1.com", "server2.com"]
      },
      SSHGroup{
          path: "hello3",
          hosts: vec!["server1.com", "server4.com"]
      }];
    let mut store = SSHStore {
       nodes: Vec::new(),
        groups: ssh_groups
    };
    store.groups_to_nodes();
    for node in store.nodes {
        if node.is_dir {
            println!("+ {}/", node.path)
        } else {
            println!("     - {}", node.name)
        }
    }
}