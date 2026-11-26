# VHS Script: VantaDB 60-Second Terminal Demo
# Render with: vhs vhs/demo.tape -> produces assets/demo.gif & assets/demo.mp4
# Replays the FROZEN run only (vhs/hero-run.json); not rendered on this
# Windows host because ttyd was unavailable.

Output assets/demo.gif
Output assets/demo.mp4

Set Shell "bash"
Set FontSize 15
Set Width 1280
Set Height 720
Set Padding 20
Set FontFamily "JetBrains Mono"
Set Theme "Catppuccin Macchiato"
Set WindowBar "Colorful"
Set LetterSpacing 0
Set LineHeight 1.2

# Scene 1: frame 1 — health check (frozen output, 12s pacing)
Type "python -c \"import json;f=json.load(open('vhs/hero-run.json'))['frames'];print('$ '+f[0]['command']);print(f[0]['output'])\""
Enter
Sleep 12s

# Scene 2: frame 2 — MCP handshake
Type "python -c \"import json;f=json.load(open('vhs/hero-run.json'))['frames'];print('$ '+f[1]['command']);print(f[1]['output'])\""
Enter
Sleep 12s

# Scene 3: frame 3 — memory capture
Type "python -c \"import json;f=json.load(open('vhs/hero-run.json'))['frames'];print('$ '+f[2]['command']);print(f[2]['output'])\""
Enter
Sleep 12s

# Scene 4: frame 4 — hybrid search and supersession
Type "python -c \"import json;f=json.load(open('vhs/hero-run.json'))['frames'];print('$ '+f[3]['command']);print(f[3]['output'])\""
Enter
Sleep 12s

# Scene 5: frame 5 — L2 scene and engine inspection
Type "python -c \"import json;f=json.load(open('vhs/hero-run.json'))['frames'];print('$ '+f[4]['command']);print(f[4]['output'])\""
Enter
Sleep 12s
