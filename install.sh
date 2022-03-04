# call the github api to get the url for the asset with the name wordle_solver from the latest release on the repository PepperLola/wordle-solver
wget -qO- https://api.github.com/repos/PepperLola/wordle-solver/releases/latest | grep browser_download_url | grep wordle_solver | cut -d '"' -f 4 | wget -i -
