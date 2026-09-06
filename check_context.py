with open(r"D:\NodeJSProject\Machenike_L16W_Center\src-tauri\target\release\machenike-l16w-center.exe", "rb") as f:
    data = f.read()

idx = 0
while True:
    pos = data.find(b"1420", idx)
    if pos == -1:
        break
    start = max(0, pos - 100)
    end = min(len(data), pos + 100)
    print("Found at pos:", hex(pos))
    print(data[start:end])
    print("-" * 50)
    idx = pos + 4
