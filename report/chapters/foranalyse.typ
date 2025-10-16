#import "@preview/cetz:0.4.2"
#import "@preview/zebraw:0.5.5": *

= Foranalyse
En database er brugt næsten alle steder ude i den rigtige verden i næsten alt
software. De bliver brugt til at opbevare en masse data, både informationer om
f.eks. kunder men også relevante informationer om de systemer som programmerne
holder i gang. Der er mange store database systemer som f.eks. PostgreSQL,
SQLite og Microsoft SQL Server. Disse tre har det til fælles at de er
relationelle databaser. Der findes dog også ikke-relationelle databaser, såsom
MongoDB.

Disse databaseformer vil der nu analyseres nærmere.

== Database typer
Der er to primære typer af database: Relationel database og en ikke-relationel
database, også kendt som en NoSQL database (Not only SQL). Disse to har hver
deres fordele og ulemper som der vil beskrives nærmere nu.

=== Relationel database
Relationelle databaser bruger primært det standarde sprog for relationelle
databaser, SQL. SQL er sprog der blev opfundet i løbet af 1970'erne til at
interegere med databaser der indeholdte strukturerede data opbygget på
relationel algebra. De bliver brugt i databaser med en tabelstruktur hvor alt
data er gemt under et kolonnenavn. Dette gør det nemt at styre hvilke data hører
til hvor. @sqlwikipedia

En stor fordel ved relationelle databaser er også at de opfylder ACID. ACID er
et sæt af egenskaber som mange relationelle databaser opfylder under deres
transaktioner. Dette gør at data kan redigeres og flyttes rundt uden anomalier
som ender med at skabe forkert data. Derfor er disse databaser brugt meget i
kommercielle programmer hvor der er brug for personstyring etc. @acidwikipedia

=== Ikke-relationel database
Ikke-relationelle databaser, modsat relationelle databaser, er ikke tvunget til
at være opsat i en tabelstruktur. Disse er databaser der typisk skal holde på
meget data som man enten er usikker på størrelsen af, eller som er i en
datastruktur som kan ændre hurtigt på sigt. De er også brugt steder hvor det er
vigtigt at man kan læse og skrive data i så hurtig en hastighed som muligt.
@whatisdatabase

Et eksempel på noget software som bruger en ikke-relationel database er ting som
Facebook og Twitter. Disse bruger databasen til at holde styr på ting som
brugergenereret indhold, kommentarer og interaktioner. Disse databaser skal
kunne skalere meget hurtigt og holde på mange forskellige typer af data, hvilket
er derfor de bliver brugt. @whatisdatabase
