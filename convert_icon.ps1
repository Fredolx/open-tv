Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile("c:\Users\admin-beats\OneDrive\xo Vibe Coding xo\iptvnator\open-tv\temp_icon.png")
$img.Save("c:\Users\admin-beats\OneDrive\xo Vibe Coding xo\iptvnator\open-tv\temp_icon_real.png", [System.Drawing.Imaging.ImageFormat]::Png)
$img.Dispose()
