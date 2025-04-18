{
    let mut txn = graph.start_txn().await.expect("Failed to start a new transaction");
    let id = uuid::Uuid::new_v4().to_string();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone())).await.unwrap();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone())).await.unwrap();
    // graph.execute(..) will not see the changes done above as the txn is not committed yet
    let mut result = graph.execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone())).await.unwrap();
    assert!(result.next().await.unwrap().is_none());
    let bookmark = txn.commit().await.unwrap();
    assert!(bookmark.is_some());
    if let Some(ref b) = bookmark {
        println!("Got a bookmark after commit: {:?}", b);
    }

    //changes are now seen as the transaction is committed.
    let mut txn = graph.start_txn_as(Operation::Read, bookmark.map(|b| vec![b])).await.expect("Failed to start a new transaction");
    let mut stream = txn.execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone())).await.unwrap();
    loop {
        let next = stream.next(txn.handle());
        if let Ok(Some(record)) = next.await {
            println!("Record: {:?}", record);
        } else {
            break;
        }
    }
}
