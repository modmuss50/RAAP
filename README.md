# RAAP - Rust ADS-B Altitude Plotter

This is a simple application to help visualise ADS-B Mode C altitude messages. The following screenshots show how the [Lockheed U-2](https://en.wikipedia.org/wiki/Lockheed_U-2) is plotted when climbing or decending from its operating altitude at 60,000ft or more.

![](https://raw.githubusercontent.com/modmuss50/RAAP/main/.github/screenshots/screenshot_1.png)
![](https://raw.githubusercontent.com/modmuss50/RAAP/main/.github/screenshots/screenshot_2.png)

This application collects data from an ADS-B reciever over the network. If you are using dump1090 run it with `--modeac`.