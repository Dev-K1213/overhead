# Privacy

Overhead has no server, no accounts, no analytics and no tracking. The developer never receives your location or anything else about you.

## What stays on your Mac

Your home location, radius and settings are saved in `~/Library/Application Support/Overhead`. Delete that folder to erase them.

## What leaves your Mac

Overhead sends your home location to these services:

- **[adsb.fi](https://adsb.fi)**, and **[adsb.lol](https://adsb.lol)** when adsb.fi doesn't respond, every 10 seconds, to find aircraft near home. **Share only an approximate location** is on by default. With it on, they get home rounded to about 1 km, not your exact address. You can change this on the location screen.
- **[OpenStreetMap Nominatim](https://osmfoundation.org/wiki/Privacy_Policy)**, only while you set home: to look up the address you type, or to show a name for the location you picked.
- **[JetAPI](https://www.jetapi.dev)**, when aircraft are nearby: Overhead sends registrations for up to eight nearby aircraft to look up JetPhotos images. Photo results are cached for six hours. If JetAPI has no result, Overhead falls back to **[Planespotters.net](https://www.planespotters.net)** when you open aircraft details and sends the aircraft's transponder hex code.
- **[ADSB.lol](https://api.adsb.lol)**, for aircraft with a callsign: Overhead sends the callsign and current coordinates to look up a possible origin and destination. Route results are cached while the app runs.
- **[ADSBDB](https://api.adsbdb.com)**, if ADSB.lol has no route match: Overhead sends the callsign to look up a possible origin and destination. Route results are cached while the app runs.
- **[Flightradar24](https://www.flightradar24.com)** and **[JetPhotos](https://www.jetphotos.com)** open only if you click their links in an aircraft's details.
- **[OpenStreetMap tile servers](https://operations.osmfoundation.org/policies/tiles/)**, when you open an aircraft's menu bar details: the map requests tiles for the aircraft and home positions so it can show both markers.

Like any website you visit, these services can see your IP address.

## Location access

If you choose **Use My Current Location**, macOS asks for permission. Overhead reads your location once, to set home, and never again. Every other time it uses the home you saved.
