
struct SSHGroup<'a> {
    path: &'a str,
    hosts: Vec<&'a str>
}
struct SSHStore<'a> {
    nodes: Vec<UITreeNode<'a>>
}
struct UITreeNode<'a> {
    path: &'a str,
    name: &'a str,
    is_dir: bool
}

impl<'a> SSHGroup<'a> {
    fn to_ui_tree_node(&self, ssh_groups: Vec<SSHGroup>) -> Vec<UITreeNode<'a>> {
        let mut nodes: Vec<UITreeNode<'a>> = vec![];
        if ssh_groups.iter().any(|x| x.path == self.path) {
            let path_node = UITreeNode{
                path: self.path,
                name: self.path,
                is_dir: true
            };
            nodes.push(path_node);
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
    },
                          SSHGroup{
                              path: "hello",
                              hosts: vec!["server1.com", "server2.com"]
                          }];
    let store = SSHStore {
        nodes: ssh_groups
            .iter()
            .map(|x| x.to_ui_tree_node())
            .flatten()
            .collect(),
    };
    for node in store.nodes {
        if node.is_dir {
            println!("+ {}/", node.path)
        } else {
            println!("     - {}", node.name)
        }
    }
}