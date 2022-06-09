# Ebina

This bot was initially made to play charades, i.e. given a set of emotes, name the series/anime. However it was expanded with aditional features like a selection of manga/anime APIs as well as WolframAlpha and Open Weather map.

Ebins supports two website interfaces, prometheus for live command metrics as well as a work in progress dashboard for controlling certain aspects. It is deployed with docker for ease of transport and supports a Postgre database for permanent storage. Only configuration info and charades is currently stored permanently.


## Connected APIs:
* Wolfram alpha
* Open weather map
* Anilist
* saucenao (With image upload)
* vndb.org (udp api, but forked and fixed some)
* Mangadex.org
* Osu (Api library written [osu_v2](https://git.fuyu.moe/Tracreed/osu_v2))
