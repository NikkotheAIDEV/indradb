use super::super::{
    BulkInsertItem, Datastore, Edge, EdgeQueryExt, SpecificEdgeQuery, SpecificVertexQuery, Transaction, Type,
    Vertex, VertexQueryExt,
};
use serde_json::Value as JsonValue;

pub fn should_bulk_insert<D: Datastore>(datastore: &mut D) {
    let vertex_t = Type::new("test_vertex_type").unwrap();
    let outbound_v = Vertex::new(vertex_t.clone());
    let inbound_v = Vertex::new(vertex_t);

    let items = vec![
        BulkInsertItem::Vertex(outbound_v.clone()),
        BulkInsertItem::Vertex(inbound_v.clone()),
    ];

    datastore.bulk_insert(items.into_iter()).unwrap();

    let edge_t = Type::new("test_edge_type").unwrap();
    let edge = Edge::new(outbound_v.id, edge_t.clone(), inbound_v.id);

    let items = vec![
        BulkInsertItem::Edge(edge.clone()),
        BulkInsertItem::VertexProperty(
            outbound_v.id,
            "vertex_property_name".to_string(),
            JsonValue::String("vertex_property_value".to_string()),
        ),
        BulkInsertItem::EdgeProperty(
            edge.clone(),
            "edge_property_name".to_string(),
            JsonValue::String("edge_property_value".to_string()),
        ),
    ];

    datastore.bulk_insert(items.into_iter()).unwrap();

    let trans = datastore.transaction().unwrap();
    let vertices = trans
        .get_vertices(SpecificVertexQuery::new(vec![outbound_v.id, inbound_v.id]))
        .unwrap();

    assert_eq!(vertices.len(), 2);
    assert_eq!(vertices[0].id, outbound_v.id);
    assert_eq!(vertices[0].t, outbound_v.t);
    assert_eq!(vertices[1].id, inbound_v.id);
    assert_eq!(vertices[1].t, inbound_v.t);

    let edges = trans.get_edges(SpecificEdgeQuery::single(edge.clone())).unwrap();

    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].outbound_id, outbound_v.id);
    assert_eq!(edges[0].t, edge_t);
    assert_eq!(edges[0].inbound_id, inbound_v.id);

    let vertex_properties = trans
        .get_vertex_properties(SpecificVertexQuery::single(outbound_v.id).property("vertex_property_name"))
        .unwrap();

    assert_eq!(vertex_properties.len(), 1);
    assert_eq!(vertex_properties[0].id, outbound_v.id);
    assert_eq!(
        vertex_properties[0].value,
        JsonValue::String("vertex_property_value".to_string())
    );

    let edge_properties = trans
        .get_edge_properties(SpecificEdgeQuery::single(edge.clone()).property("edge_property_name"))
        .unwrap();

    assert_eq!(edge_properties.len(), 1);
    assert_eq!(edge_properties[0].edge, edge);
    assert_eq!(
        edge_properties[0].value,
        JsonValue::String("edge_property_value".to_string())
    );
}

// Bulk insert allows for redundant vertex insertion
pub fn should_bulk_insert_a_redundant_vertex<D: Datastore>(datastore: &mut D) {
    let vertex_t = Type::new("test_vertex_type").unwrap();
    let vertex = Vertex::new(vertex_t);

    let trans = datastore.transaction().unwrap();
    assert!(trans.create_vertex(&vertex).unwrap());

    let items = vec![BulkInsertItem::Vertex(vertex)];
    assert!(datastore.bulk_insert(items.into_iter()).is_ok());
}

// As an optimization, bulk insert does not verify that the vertices
// associated with an inserted edge exist; this verifies that
pub fn should_bulk_insert_an_invalid_edge<D: Datastore>(datastore: &mut D) {
    let vertex_t = Type::new("test_vertex_type").unwrap();
    let v1 = Vertex::new(vertex_t.clone());
    let v2 = Vertex::new(vertex_t);

    let trans = datastore.transaction().unwrap();
    assert!(trans.create_vertex(&v1).unwrap());

    let edge_t = Type::new("test_edge_type").unwrap();

    let items = vec![BulkInsertItem::Edge(Edge::new(v1.id, edge_t.clone(), v2.id))];
    assert!(datastore.bulk_insert(items.into_iter()).is_ok());
    let items = vec![BulkInsertItem::Edge(Edge::new(v2.id, edge_t.clone(), v1.id))];
    assert!(datastore.bulk_insert(items.into_iter()).is_ok());
}
