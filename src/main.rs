use rustrbx::AsyncTimeline;

#[tokio::main]
async fn main() -> rustrbx::Result<()> {
    // Should we rely on references here or should we just copy memory here?
    //let url = &"https://users.roblox.com/v1/users/search?keyword=aaaa".to_string();
    //let cursor = &"eyJzdGFydEluZGV4IjoxMDAsImRpc2NyaW1pbmF0b3IiOiJrZXl3b3JkOmFhYWEiLCJjb3VudCI6MTAwfQozNGUwYTAwYjhjOTgzZjUzYWUwMDA5ZDIzZTUwZDU3Y2MxMDFkOGI4NWJmMmQwNzBkMzZlNTZkNmRlODQ2MWUy".to_string();
    //let pointed = rustrbx::point::<Vec<rustrbx::users::UserQuery>>(url, cursor).await?;
    //let query = pointed.timeline();
    //println!("{:#?}", query.current().data);
    //println!("new: \n {:#?}", query.forward().await?.data);
    let query_results = rustrbx::users::search("test".to_string()).await?;
    println!("{:#?}", query_results.data);
    let timeline = query_results.timeline();
    println!("{:#?}", timeline.forward().await?.data);
    println!("{:#?}", timeline.backwards().await?.data); // fail here
    Ok(())
}