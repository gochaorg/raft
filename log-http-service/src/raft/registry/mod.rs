#[test]
fn registry_test() {
    use local_ip_address::local_ip;
    let my_local_ip = local_ip().unwrap();
    println!("This is my local IP address: {:?}", my_local_ip);

    //my_local_ip.is
}