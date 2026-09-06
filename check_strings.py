with open(r"D:\NodeJSProject\Machenike_L16W_Center\src-tauri\target\release\machenike-l16w-center.exe", "rb") as f:
    data = f.read()

print("src-tauri release exe:")
print("Contains http://localhost:1420:", b"http://localhost:1420" in data)
print("Contains localhost:1420:", b"localhost:1420" in data)
print("Contains index-BuGJCIjH.css:", b"index-BuGJCIjH.css" in data)
print("Contains MACHENIKE:", b"MACHENIKE" in data)
