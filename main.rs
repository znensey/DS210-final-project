use std::{error::Error, collections::HashMap};
use csv::Reader;
use petgraph::graph::NodeIndex;

mod graph;
mod centrality;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct Item {
    customer_id: usize,
    age: usize,
    gender: bool,
    item_purchased: String,
    category: String,
    purchase_amount: usize,
    location: String,
    size: String,
    color: String,
    season: String,
    review_rating: usize,
    subscription_status: bool,
    shipping_type: String,
    discount_applied: bool,
    promo_code_used: bool,
    previous_purchases: usize,
    payment_method: String,
    preferred_payment_method: String,
    frequency_of_purchases: String,
    edges: Vec<String>,
}

fn read_csv(file_path: &str) -> Result<Vec<Item>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let _headers = reader.headers()?.clone();

    let data: Vec<Item> = reader
        .records()
        .filter_map(|result| {
            result.ok().and_then(|record| {
                Some(Item {
                    customer_id: record[0].parse().unwrap_or_default(),
                    age: record[1].parse().unwrap_or_default(),
                    gender: record[2].parse().unwrap_or(false),
                    item_purchased: record[3].to_string(),
                    category: record[4].to_string(),
                    purchase_amount: record[5].parse().unwrap_or_default(),
                    location: record[6].to_string(),
                    size: record[7].to_string(),
                    color: record[8].to_string(),
                    season: record[9].to_string(),
                    review_rating: record[10].parse().unwrap_or_default(),
                    subscription_status: record[11].parse().unwrap_or_default(),
                    shipping_type: record[12].to_string(),
                    discount_applied: record[13].parse().unwrap_or_default(),
                    promo_code_used: record[14].parse().unwrap_or_default(),
                    previous_purchases: record[15].parse().unwrap_or_default(),
                    payment_method: record[16].to_string(),
                    preferred_payment_method: record[17].to_string(),
                    frequency_of_purchases: record[18].to_string(),
                    edges: Vec::new(),
                })
            })
        })
        .collect();

    Ok(data)
}

fn main() {
    match read_csv("shopping_trends.csv") {
        Ok(items) => {
            let (graph, item_node_mapping) = graph::build_graph(&items);
            let degree_centrality = centrality::calculate_degree_centrality(&graph);

            let reverse_mapping: HashMap<NodeIndex, String> = item_node_mapping
                .iter()
                .map(|(item, &node)| (node, item.clone()))
                .collect();

            for node in graph.nodes() {
                if let Some(item_name) = reverse_mapping.get(&node) {
                    let centrality = degree_centrality[node.index()];
                    println!("Item '{}': Degree Centrality: {:.4}", item_name, centrality);
                }
            }

            let seasonal_centrality =
                centrality::calculate_seasonal_degree_centrality(&graph, &items, &item_node_mapping);

            for (season, centrality_scores) in seasonal_centrality.iter() {
                println!("Season {}:", season);
                for (node, centrality) in graph.nodes().zip(centrality_scores.iter()) {
                    if let Some(item_name) = reverse_mapping.get(&node) {
                        println!(
                            "  Item '{}': Seasonal Degree Centrality: {:.4}",
                            item_name, centrality
                        );
                    }
                }
            }
        }
        Err(e) => println!("Error reading CSV file: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph;
    use crate::centrality;



    fn sample_items() -> Vec<Item> {
        vec![
            Item {
                customer_id: 1,
                item_purchased: "T-shirt".to_string(),
                category: "Clothing".to_string(),
                season: "Summer".to_string(),
                ..Default::default()
            },
            Item {
                customer_id: 2,
                item_purchased: "Jeans".to_string(),
                category: "Clothing".to_string(),
                season: "Winter".to_string(),
                ..Default::default()
            },
            Item {
                customer_id: 3,
                item_purchased: "Sunglasses".to_string(),
                category: "Accessories".to_string(),
                season: "Summer".to_string(),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn test_seasonal_centrality() {
        let items = sample_items();
        let (graph, item_mapping) = graph::build_graph(&items);

        let seasonal_centrality =
            centrality::calculate_seasonal_degree_centrality(&graph, &items, &item_mapping);

        assert!(seasonal_centrality.contains_key("Summer"));
        assert!(seasonal_centrality.contains_key("Winter"));

        let summer_scores = seasonal_centrality.get("Summer").unwrap();
        assert_eq!(summer_scores.len(), 2); // T-shirt and Sunglasses

        let winter_scores = seasonal_centrality.get("Winter").unwrap();
        assert_eq!(winter_scores.len(), 1); // Jeans
    }

    #[test]
    fn test_duplicate_items_handling() {
        let mut items = sample_items();
        // Add duplicate items
        items.push(Item {
            customer_id: 4,
            item_purchased: "T-shirt".to_string(),
            category: "Clothing".to_string(),
            season: "Summer".to_string(),
            ..Default::default()
        });

        let (graph, item_mapping) = graph::build_graph(&items);

 
        assert_eq!(graph.node_count(), 3); 
        assert!(item_mapping.contains_key("T-shirt"));
        assert!(item_mapping.contains_key("Jeans"));
    }

    #[test]
    fn test_no_edges_with_different_categories() {
        let items = vec![
            Item {
                item_purchased: "T-shirt".to_string(),
                category: "Clothing".to_string(),
                ..Default::default()
            },
            Item {
                item_purchased: "Shoes".to_string(),
                category: "Footwear".to_string(),
                ..Default::default()
            },
        ];

        let (graph, _) = graph::build_graph(&items);

     
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);
    }
}
