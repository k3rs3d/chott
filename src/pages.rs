use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub template: String,
    pub connections: Vec<PageConnection>,
    pub title: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PageId(pub String);

impl From<&str> for PageId {
    fn from(s: &str) -> Self {
        PageId(s.to_owned())
    }
}
impl std::fmt::Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageConnection {
    pub name: String,   // e.g., "north", "down", "to city gate", "forwards"
    pub target: PageId, // page id (slug) you go to if you click this
}

// PageGraph is a HashMap keyed by id
pub type PageGraph = HashMap<PageId, Page>;

pub fn load_page_graph() -> PageGraph {
    let mut graph = PageGraph::new();

    graph.insert(
        PageId::from("small-town"),
        Page {
            id: PageId::from("small-town"),
            template: "small-town.html".to_string(),
            connections: vec![PageConnection {
                name: "North".to_string(),
                target: PageId::from("route-1"),
            }],
            title: "Small Town".to_string(),
            description: "A quiet, peaceful town.".to_string(),
            metadata: HashMap::new(),
        },
    );

    graph.insert(
        PageId::from("route-1"),
        Page {
            id: PageId::from("route-1"),
            template: "route-1.html".to_string(),
            connections: vec![
                PageConnection {
                    name: "North".to_string(),
                    target: PageId::from("green-city"),
                },
                PageConnection {
                    name: "South".to_string(),
                    target: PageId::from("small-town"),
                },
            ],
            title: "Route 1".to_string(),
            description: "A winding route with tall grass and wild things.".to_string(),
            metadata: HashMap::new(),
        },
    );

    graph.insert(
        PageId::from("green-city"),
        Page {
            id: PageId::from("green-city"),
            template: "green-city.html".to_string(),
            connections: vec![PageConnection {
                name: "South".to_string(),
                target: PageId::from("route-1"),
            }],
            title: "Green City".to_string(),
            description: "A bustling city under the old trees.".to_string(),
            metadata: HashMap::new(),
        },
    );

    graph
}

/// requested_connection = the user's POSTed button direction name ("north" etc)
pub async fn valid_move<'a>(
    current_page_id: &'a PageId,
    requested_connection: &'a str,
    pages: &'a PageGraph,
) -> Option<&'a PageConnection> {
    pages.get(current_page_id).and_then(|page| {
        page.connections
            .iter()
            .find(|conn| conn.name == requested_connection)
    })
}
