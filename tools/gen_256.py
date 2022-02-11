
for i in range(0, 256):
    print(f"\tColor24::new((({i} >> 5) & 0b111) * 36, (({i} >> 2) & 0b111) * 36, (({i}) & 0b11) * 81),")