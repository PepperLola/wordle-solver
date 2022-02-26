words = []
with open('words.txt', 'r') as f:
    words = f.read().split('\n')

INCORRECT = 0
WRONG_POSITION = 1
CORRECT = 2

pattern = [INCORRECT, INCORRECT, CORRECT, INCORRECT, CORRECT]
test = 'amaze'

present = {} # letter to list of idx it isn't

for i in range(0, 5):
    maxIdx = len(words) - 1
    j = 0
    if pattern[i] == WRONG_POSITION:
        if test[i] in present.keys():
            present[test[i]].push(i)
        else:
            present[test[i]] = [i]
    while j < maxIdx:
        if pattern[i] == WRONG_POSITION and (words[j][i] == test[i] or not test[i] in words[j]):
            del words[j]
            j -= 1
            maxIdx -= 1
        j += 1

for i in range(0, 5):
    maxIdx = len(words) - 1
    j = 0
    while j < maxIdx:
        if pattern[i] == INCORRECT and words[j][i] == test[i] and words[j][i] in present.keys() and i in present[words[j][i]]:
            del words[j]
            j  -= 1
            maxIdx -= 1
        j += 1


print(words)
