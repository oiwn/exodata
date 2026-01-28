# A lot of duplicates in database

We have st_refname with the reference source. So

 - HAT-P-35 (main star) - 7 different measurements from different paper
 - HAT-P-35 B - companion star (Ngo et al. 2016)
 - HAT-P-35 C - companion star (Stassun et al. 2019)
 - The 23.56 M☉ is from Stassun et al. 2017 - clearly erroneous (data)

How to handle this? Some thoughts:

  1. Table view: Could filter to show only one row per system (maybe the discovery paper, or a preferred source)
  2. Detail page: Show all measurements as a table with their sources - this is actually valuable scientific info
  3. Companions: Either show them as separate entries or group them under the main host

