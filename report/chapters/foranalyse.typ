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

== Database struktur
At gemme, læse og skrive så store mængder data kan være svært og kræver en god
datastruktur. Disse strukturer er typisk baseret på en binær træstruktur. Det
normale binære søgtræ har dog den ulempe at den ikke er særlig skalerbar, da den
i længden kan blive langsom i forhold til andre træstrukturer. Der er derfor
blevet udviklet andre former for træstrukturer som skalere bedre i forhold til
læse- og skrivehastighed. Det normale binære søgetræ vil nu analyseres.

=== Binary Search Tree
På @btree kan et binært søgetræ ses.

#let btree = cetz.canvas({
  import cetz.draw: *

  circle((3, 4), name: "7", radius: 0.5)
  content((name: "7"), [7])

  circle((1, 2), name: "5", radius: 0.5)
  content((name: "5"), [5])
  line("7", "5")

  circle((5, 2), name: "11", radius: 0.5)
  content((name: "11"), [11])
  line("7", "11")

  circle((2, 0), name: "6", radius: 0.5)
  content((name: "6"), [6])
  line("5", "6")

  circle((0, 0), name: "4", radius: 0.5)
  content((name: "4"), [4])
  line("5", "4")

  circle((6, 0), name: "13", radius: 0.5)
  content((name: "13"), [13])
  line("11", "13")

  circle((4, 0), name: "9", radius: 0.5)
  content((name: "9"), [9])
  line("11", "9")

  circle((5, -2), name: "10", radius: 0.5)
  content((name: "10"), [10])
  line("9", "10")

  circle((3, -2), name: "8", radius: 0.5)
  content((name: "8"), [8])
  line("9", "8")
})

#figure(
  rect(inset: 10pt, [#btree]), caption: "Binært søgetræ med værdierne 4, 5, 6, 7, 8, 9, 10, 11 og 13",
) <btree>

Det binære søgetræ er en træstruktur hvor alle keys er større end deres venstre
undertræ og mindre end deres højre undertræ. Dette gør at den har relativt
hurtige insert, update og removal hastigheder end f.eks. en singly-linked list.
Tidskompleksiteten af et binært søgetræ kan ses på @bst-timecomplexity.

#figure(
  scale(
    90%, reflow: true, table(
      columns: (auto, auto, auto), align: horizon, inset: 10pt, table.header(
        [*Operation*], [*Average*], [*Worst Case*], [*Search*], [$O(h)$], [$O(n)$], [*Insert*], [$O(h)$], [$O(n)$], [*Delete*], [$O(h)$], [$O(n)$],
      ),
    ),
  ), caption: [Tidskompleksiteten af et BST i Big O notation, hvor _n_ er antallet af noder i
    træet og _h_ er højden af træet],
) <bst-timecomplexity>

Det kan ses i ovenstående tabel at den gennemsnitlige tidskompleksitet af et
binært søgetræ er $O(h)$, altså højden af træet. Dette er hurtigere end en
usorteret liste, hvilket gør at det er bedre at køre operationer mod. Dog har
det binøre søgetræ den ulempe at der er mulighed for at tidskompleksiteten kan
nå $O(n)$ hvis det ikke bliver balanceret. Dette skyldes at et ubalancaret
binært søgetræ kan være "skævt", hvilket vil sige at det enten kun går til
venstre eller højre.

På grund af det normale binære søgetræs dårlige tidskompleksiteter er derfor
fundet bedre metoder at lave træstrukturer på kaldet selvbalancerede træer.
Disse vil nu analyseres.

// Dog kan det også ses at det i værste tilfælde kan// have en tidskompleksitet på $O(n)$, hvilket er på niveau med en usorteret liste.
// Dette er ikke optimalt, da en database gerne skal kunne køre operationer i en
// meget hurtig hastighed.
//
=== Self-balanced Binary Search Tree

*Kig i bogen for det her*

På @bplustree Kan et B+ Tree ses.

#let btree = cetz.canvas(
  {
    import cetz.draw: *

    // ----------ROOT-------------
    rect((0, 0), (rel: (3, -1)), fill: luma(200))
    grid((0, 0), (rel: (3, -1)))

    // root keys
    content((0.5, -0.5), [#text(size: 16pt, [3])])
    content((1.5, -0.5), [#text(size: 16pt, [5])])

    //children
    rect((0, -1), (rel: (.75, -.5)), name: "root1")
    rect((.75, -1), (rel: (.75, -.5)), name: "root2")
    rect((1.5, -1), (rel: (.75, -.5)), name: "root3")
    rect((2.25, -1), (rel: (.75, -.5)))
    circle("root1", radius: (.07), fill: black)
    circle("root2", radius: (.07), fill: black)
    circle("root3", radius: (.07), fill: black)

    // --------LEFT CHILD NODE-----------
    // keys
    rect((-4, -3), (rel: (3, -1)), fill: luma(200))
    grid((-4, -3), (rel: (3, -1)))
    content((-3.5, -3.5), [#text(size: 16pt, [1])])
    content((-2.5, -3.5), [#text(size: 16pt, [2])])
    rect((-4, -3), (rel: (1, -1)), name: "lmain")

    // children
    rect((-4, -4), (rel: (.75, -.5)), name: "l1")
    rect((-3.25, -4), (rel: (.75, -.5)), name: "l2")
    rect((-2.5, -4), (rel: (.75, -.5)), name: "l3")
    rect((-1.75, -4), (rel: (.75, -.5)), fill: red, name: "l4")

    circle("l1", radius: (.07), fill: black)
    circle("l2", radius: (.07), fill: black)
    circle("l4", radius: (.07), fill: black)

    // --------MIDDLE CHILD NODE-----------
    // keys
    rect((0, -3), (rel: (3, -1)), fill: luma(200))
    grid((0, -3), (rel: (3, -1)))
    content((.5, -3.5), [#text(size: 16pt, [3])], name: "mmain")
    content((1.5, -3.5), [#text(size: 16pt, [4])])

    // children
    rect((0, -4), (rel: (.75, -.5)), name: "m1")
    rect((.75, -4), (rel: (.75, -.5)), name: "m2")
    rect((1.5, -4), (rel: (.75, -.5)), name: "m3")
    rect((2.25, -4), (rel: (.75, -.5)), fill: red, name: "m4")

    circle("m1", radius: (.07), fill: black)
    circle("m2", radius: (.07), fill: black)
    circle("m4", radius: (.07), fill: black)

    // --------RIGHT CHILD NODE-----------
    // keys
    rect((4, -3), (rel: (3, -1)), fill: luma(200))
    grid((4, -3), (rel: (3, -1)))
    content((4.5, -3.5), [#text(size: 16pt, [5])])
    content((5.5, -3.5), [#text(size: 16pt, [6])])
    content((6.5, -3.5), [#text(size: 16pt, [7])])

    // children
    rect((4, -4), (rel: (.75, -.5)), name: "r1")
    rect((4.75, -4), (rel: (.75, -.5)), name: "r2")
    rect((5.5, -4), (rel: (.75, -.5)), name: "r3")
    rect((6.25, -4), (rel: (.75, -.5)))

    circle("r1", radius: (.07), fill: black)
    circle("r2", radius: (.07), fill: black)
    circle("r3", radius: (.07), fill: black)

    // -----------LINES----------
    line("root1", (-3.5, -2), (-3.5, -3), mark: (end: ">", fill: black))
    line("root2", (0.5, -3), mark: (end: ">", fill: black))
    line((1.9, -1.5), (4.5, -2.5), (4.5, -3), mark: (end: ">", fill: black))

    line((-1, -4.25), (0, -3.5), mark: (end: ">", fill: black))
    line((3, -4.25), (4, -3.5), mark: (end: ">", fill: black))
  },
)

#figure(
  rect(btree, inset: 15pt), caption: [Et B+ Tree med værdierne fra 1 til 7],
) <bplustree>

Jeg finder information om B+ Tree fra @databaseinternals.

== Læsning fra disk
For at læse og skrive data fra og til disken, skal det data først ligge i
hukommelsen af computeren. Hukommelsen læser disken i én "page" ad gangen,
hvilket er 4096 bytes på de fleste styresystemer @pagewikipedia . Dette betyder
at ens kode skal optimeres ved kun at læse de data som der er relevante for den
operation der bliver kørt mod databasen, én page ad gangen.

- Side fra bogen hvor de snakker om pages
- Hvorfor pages?

== Programmering
Mange databaser er i dag skrevet i programmeringssprogene C og C++. Disse sprog
er "low-level", forstået som at man arbejder tæt med hardwaren af computeren.
Dette er f.eks. ved at man selv skal allokere hukommelse dynamisk. Dette gør
også at ting kan optimeres rigtig meget, da sprogene ikke selv bruger en
garbage-collector til at sørge for at der ikke er nogle memory fejl, og ikke
håndtere det i runtime.

De mest hyppige memory bugs vil nu beskrives, og nogle kodeeksempler der udløser
fejlen vil vises. Kodeeksemplerne vil blive skrevet i C.

=== Memory Leak
Et memory leak opstår når der bliver allokeret memory uden at det bagefter
bliver frigivet. Dette kan gøres ved at lave en pointer og så give den en værdi.
Dette vil medføre at det memory er optaget indtil slutningen af programmet.

Koden i @mallocfunctions viser et meget overdrevet eksempel på et memory leak.

#figure([
```c
int main() {
    while(true) {
        leak_memory(100);
    }

    return 0;
}

void leak_memory(int i) {
    int *ptr = malloc(sizeof(int));
    *ptr = i;
    return;
}
```
], caption: [En funktion i C der allokerer memory kontinuerligt]) <mallocfunctions>

Hvis man er meget uopmærksom på sine pointers og ikke holder styr på at frigive
dem efter de er brugt, ender det ofte i et memory leak. Disse kan ende med at
være dyre, da de vil have en virkning på ydeevne, og i værste tilfælde vil
crashe programmet.

=== Use-After-Free

#figure([
```c
              int main() {
                  int *ptr = malloc(sizeof(int));
                  *ptr = 10;
                  free(ptr);
                  do_something(ptr);
                  return 0;
              }

              void do_something(int *ptr) {
                  // does something with the pointer ...
              }
              ```
], caption: [Et program der bruger memory efter det er frigivet])

#pagebreak()
=== Buffer Overflow

#figure([
```c
#include <stdio.h>

int main() {
    char buf[64] // create a buffer that holds 64 characters
    gets(buf); // get input from user

    return 0;
}
```
], caption: [Et program der kan lave et buffer overflow])

#pagebreak()
=== Data Race

#figure([
```c
#include <pthread.h>

int main() {
    // define 2 threads
    pthread_t thread1;
    pthread_t thread2;

    int shared_value = 0;

    // make the threads do some operation on a shared value
    pthread_create(&thread1, NULL, foo, &shared_value);
    pthread_create(&thread2, NULL, bar, &shared_value);

    // wait for threads to finish
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);

    return 0;
}

void* foo(void* arg) {
    // read and write continually to and from arg ...
}

void* bar(void* arg) {
    // read and write something else continually to and from arg ...
}
```
], caption: [Et program der viser et data race])

#pagebreak()
=== Noter

- Databaser
- Problemer i databaser (skrevet i C, C++, sikkerhed i memory)
  - SQL Server (C og C++)
  - PostgreSQL (C og C++)
  - MySQL (C og C++),
  - MongoDB (C, C++, JavaScript og Python(?) )
  - SQLite (C)
- Strukturen af databaser måske?

```rust
fn main() {
    println!("lol");

    for i in 0..100 {
      println!("lol {i}")
    }
}
```
