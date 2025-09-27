#import "src/project.typ": *

#show: body => project(
  title: "Dokumentbaseret database", subtitle: "Implementering og optimering af en dokumentbaseret database", authors: ("Jonas Vittrup Biegel",), supervisors: ("Brian Hvarregaard",), theme: "Semesterprojekt", projectperiod: "Efterårssemstret", group-number: "Gruppe 9", rev-number: "1", resume: [
    #lorem(100)
  ], characters: 2083, // unfortunately i couldnt find a good way to this automatically
  body,
)

// main matter
#include "chapters/indledning.typ"
#include "chapters/foranalyse.typ"
#include "chapters/kravsspecifikation.typ"
#include "chapters/problemformulering.typ"
#include "chapters/metode.typ"
#include "chapters/analyse.typ"
#include "chapters/design.typ"
#include "chapters/implementering.typ"
#include "chapters/test.typ"
#include "chapters/overdragelse.typ"
#include "chapters/konklusion.typ"

// appendix
// make a appendix file and include it here
