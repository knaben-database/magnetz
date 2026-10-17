# Magnetz

Magnetz is used as the backbone to check the status and health of magnets for Knaben Database.

It relies on a 3rd party ([Torrust-tracker](https://github.com/torrust/torrust-tracker)) to grab the magnet values.

Then it uses Redis to look up the "heatmap" generated from Knaben web. Basically, once a listing is seen, it will increment the listings in Redis making it more likely to be checked next by Magnetz.

Lastly, it will update the results to Elasticsearch, making them public to everyone.
