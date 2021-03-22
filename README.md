<h2>
    Implementation of cards against humanity for online gaming
</h2>
<p>
    in order to run this program type ./run.sh it should automatically install cargo and run the program however this run script only works on debian based systems at it attempts to use apt to install apache2.

    the things needed to run this program are a web server, and cargo
</p>

<h2>
    running code and examples
</h2>
<p>
    if you are on windows google download rust lang/download cargo and install using the exe they provide

    if on linux curl https://sh.rustup.rs | sh

    to run the example once cargo is installed clone the repo using either a graphical git client or installing it with your distros package manager chocolatey for windows, apt for debian, brew for mac, pacman for arch swupd for clear linux, pkg for redox, yum for SUSE (i think)

    once downloaded use git clone $url_of_repo, cd cah-backend, cargo run --example randomgen, or cargo run and firefox/chrome game.html
</p>
