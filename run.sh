cd cah-backend
curl https://sh.rustup.rs | sh
export PATH=$PATH:~/.cargo/bin/
sudo apt-get install apache2
sudo cp html/* /var/www/html
cargo run