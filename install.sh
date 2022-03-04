wget -qO- https://api.github.com/repos/PepperLola/wordle-solver/releases/latest | grep browser_download_url | grep wordle_solver | cut -d '"' -f 4 | wget -i -
mkdir $HOME/.wordle_solver && curl https://gist.githubusercontent.com/PepperLola/bbc3512ba937a56572219536b60da4fd/raw/1295fc03cabfcc7d2bd290dc75fc28a8569dc3fe/words.txt > $HOME/.wordle_solver/words.txt
