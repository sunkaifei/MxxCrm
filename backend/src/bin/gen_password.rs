use bcrypt::{hash, verify, DEFAULT_COST};

fn main() {
    let password = "123456";
    let hashed = hash(password, DEFAULT_COST).unwrap();
    println!("Hash: {}", hashed);
    println!("Length: {}", hashed.len());
    println!("Verify: {}", verify(password, &hashed).unwrap());
    
    // Also verify the Python-generated hash
    let python_hash = "$2b$10$ddekNYPqokhs.N77STQjeOLE6vmJKOo95G.1yJc4CHPCVC9vZI/O2";
    println!("Python hash verify: {}", verify(password, python_hash).unwrap_or(false));
}
