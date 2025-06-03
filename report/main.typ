#import "src/project.typ": *

#show: body => project(
  title: "Database i Rust",
  subtitle: "Implementering og optimering af en Database i Rust",
  authors: (
    "Jonas Vittrup Biegel",
  ),
  supervisors: (
    "Vejleder 1",
    "Vejleder 2"
  ),
  theme: "Semesterprojekt",
  projectperiod: "Efterårssemstret",
  group-number: "Gruppe X",
  rev-number: "1",
  resume: [
    #lorem(100)
  ],
  characters: 2083, // unfortunately i couldnt find a good way to this automatically
  body
)

// main matter
#include "chapters/indledning.typ"
#include "chapters/redegoerelse.typ"
#include "chapters/analyse.typ"
#include "chapters/implementering.typ"
#include "chapters/diskussion.typ"
#include "chapters/konklusion.typ"

// bibliography

// appendix
  // make a appendix file and include it here
