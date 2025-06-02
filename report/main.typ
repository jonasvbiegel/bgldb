#import "src/project.typ": *

#show: body => project(
  title: "UCN Rapport",
  subtitle: "UCN Rapport skrevet i Typst",
  authors: (
    "Jonas",
    "Hans",
    "Bob",
  ),
  supervisors: (
    "Lars",
    "Mis"
  ),
  theme: "Semesterprojekt",
  projectperiod: "Forårssemestret",
  group-number: "Gruppe 69",
  rev-number: "1",
  resume: [
    #lorem(100)
  ],
  characters: 2083, // unfortunately i couldnt find a good way to this automatically
  body
)

// main matter
#include "chapters/introduktion.typ"
#include "chapters/eksempler.typ"
#include "chapters/konklusion.typ"

// bibliography

// appendix
  // make a appendix file and include it here
