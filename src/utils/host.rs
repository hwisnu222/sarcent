pub fn get_host(ip: &String) -> String{
    if !ip.contains("http://") || !ip.contains("https://"){
        return format!("http://{}", ip)
    };

    return ip.to_string()
}
