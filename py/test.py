import math

for i in range(15,128):
    t = i / 2 * math.sqrt(3)
    if abs(1 - (t - math.floor(t))) < 0.01:
        print(i, t)
