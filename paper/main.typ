#import "/github/typst-templates/packages/slides/lib.typ": *

#show: slides.with(
  title: "IFT6251",
  subtitle: [Evolving _Glitch Art_ - Eelco den Heijer],
  footer: none,

  author: "Etienne Collin",
  email: none,
  semester: [Hiver 2025],
  date: datetime.today().display(),
  lang: "fr",

  show-subtitle-slide: true,
  show-author: true,
  show-outline: true,
  show-footer: true,
  show-page-numbers: true,

  outline-text: "Table des Matières",
  font: "CommitMono Nerd Font",
  header-font: "Glitch Goblin",
)

#slide[
  #show: align.with(center + horizon)
  #heading()[Origines]
]

#slide[
  #show: align.with(horizon)
  #grid(
    columns: 2,
    [
      + _Circuit bending_
      + _Glitch music_
        - #link("https://www.youtube.com/watch?v=JZcMLjnm1ps")[Ryoji Ikeda]
      + _Glitch art_
        - _Wordpad effect_
    ],
    [


      #figure(image("assets/ikeda.png", width: 80%))
      #align(center)[@rugoffRyojiIkedaReleasing2021]
    ],
  )
]

#slide[
  #show: align.with(center + horizon)
  #heading(
    level: 2,
  )[Exemples de _Circuit Bending_ @elonmuscular_420HelpMeFind2025]

  #grid(
    columns: 2,
    figure(image("assets/bending_1.png", width: 80%)),
    figure(image("assets/bending_2.png", width: 80%)),

    figure(image("assets/bending_3.png", width: 80%)),
    figure(image("assets/bending_4.png", width: 80%)),

    figure(image("assets/bending_5.png", width: 80%)),
  )
]


#slide[
  #show: align.with(center + horizon)
]

#slide[
  = Le _Glitch Art_, c'est quoi?
  #show: align.with(horizon)

  - Musique
  - Animation
  - Rendus photoréalistes
  - Peinture
  - Littérature
  - _Bots_
    - Ben Baker-Smith
    - #link("https://www.flickr.com/people/glitchbot/")[_GlitchBot_]
]

#slide[
  == Caractéristiques du _Glitch Art_
  - Possède une complexité
  - Production pseudo-aléatoire#super[\*\*\*]
  - _Glitch_ déterministes


  #show: align.with(bottom)

  #text(size: 16pt)[#super[\*\*\*]Je ne suis pas entièrement d'accord]
]

#slide[
  #show: align.with(horizon)
  #quote()[
    _[...] one can not create a glitch image, one can merely trigger a glitch, and this volatile nature of glitch art makes it a pseudo-aleatoric art form. Applying a glitch operation to an image is very simple, but creating interesting visual content is far from trivial. [...] Although finding interesting visual content using Glitch is difficult, it is by no means a random process. Applying the same glitch operations on the same image will result in the same end image._
  ]
]


#slide[
  #show: align.with(center + horizon)
  == Est-ce que _Glitch_ est le bon terme?
]

#slide[
  = Evolving _Glitch Art_ - Objectifs

  - Génotype de l'art avec opérateurs génétiques
    - (initialisation, crossover, mutation)?
  - Taux de fatalité
  - Impact visuel
  - Contribution à la diversité de l'art


  #v(-2em)
  #show: align.with(horizon)
  #quote(
    attribution: [E. den Heijer @denheijerEvolvingGlitchArt2013],
  )[_[...] can we evolve aesthetically pleasing images that are different from images that we know from existing evolutionary art systems?_]
]

#slide[
  == Evolutionary Art
  #show: align.with(horizon)

  #quote(
    attribution: [E. den Heijer @denheijerEvolvingGlitchArt2013],
  )[_[...] a field where methods from Evolutionary Computation are used to create works of art._]

  #v(-1em)

  #quote(
    attribution: [Wikipedia @EvolutionaryArt2025],
  )[_[...] a branch of generative art, in which the artist does not do the work of constructing the artwork, but rather lets a system do the construction._]
]

#slide[
  == Types d'images étudiés
  + Windows Bitmap (bmp)
  + gif
  + jpeg
  + raw (uncompressed raw image data)
  + png
  + Compressed tiff.

  #show: align.with(center + horizon)
  #text(size: 22pt)[Format différent $attach(==>, t: "Op")$ Effet différent]

  #text(size: 22pt)[Format non-compressé $==>$ Plus stable]

  #text(size: 22pt)[Data vs. Metadata]
]

#slide[
  == _Glitch Recipes_

  #show: align.with(center + horizon)
  #show: text.with(size: 22pt)
  #grid(
    columns: 2,
    align: left + top,
    column-gutter: 1fr,
    [
      *Fonctions*
      - `insert`
      - `delete`
      - `replace`
      - `and`
      - `or`
      - `xor`
      - `not`
      - `reverse`
      - `setImageFormat`
    ],
    [

      *Arguments*
      - Relatifs à la taille de l'image en `Bytes`
      - `position` $in [0, 1]$
        - Initialisation $in [0.02, 1]$
      - `size` $in [10^(-4), 10^(-2)]$
    ],
  )
]

#slide[
  #show: align.with(center + horizon)
  #show: text.with(size: 13pt)
  #v(-4em)
  #table(
    columns: 4,
    inset: 6pt,
    [Operations], [Argument 1], [Argument 2], [Description],
    [`insert`],
    [`position`],
    [`random bytes` of length $N$ \ ($N in [2, 64]$)],
    [Inserts random bytes at a certain position],

    [`delete`],
    [`position`],
    [`size`],
    [Deletes $N$ bytes from a certain position where $N="size"dot"imagesize"$],

    [`replace`],
    [`byte1`],
    [`byte2`],
    [Replaces every occurence of `byte1` with `byte2`],

    [`and`, `or`, `xor`],
    [`position`],
    [`bit mask` of length $N$ \ ($N in [2, 64]$)],
    [Performs a binary operation at a certain position using the bitmask],

    [`not`],
    [`position`],
    [`size`],
    [Inverts $N$ bytes starting at a certain position, where \ $N="size"dot"imagesize"$],

    [`reverse`],
    [`position`],
    [`size`],
    [Reverses $N$ bytes from a certain position, where \ $N="size"dot"imagesize"$],

    [`setImageFormat`],
    [`format`],
    [-],
    [Saves the current image in the specified format, and reads the binary data from the new format.],
  )
]

#slide[
  #show: align.with(center + horizon)
  #v(-1.5em)
  #figure(
    image("assets/genetic_glitch_1.png", height: 110%),
  ) <fig-genetic_glitch_1>

]

#slide[
  = Méthodologie

  #show: text.with(size: 20pt)
  Jeu de données de 500 images

  *1. Initialisation*
  - Générer 1 à 5 opérations

  #show: later

  *2. _Crossover_*
  - Séléctionner 2 parents
  - Sélectionner l'image d'un des deux parents aléatoirement
  - Séparer les opérations des deux parents en deux groupes
  - Fusionner le groupe 1 d'un parent avec le groupe 2 de l'autre
]

#slide[
  #heading(level: 1, outlined: false)[Méthodologie]
  #show: text.with(size: 20pt)
  Jeu de données de 500 images

  *3. Mutation*
  - Changer la source d'une image ($P=0.1$)
  - Remplacer une opération par une nouvelle opération ($P=0.1$)
  - Altérer un argument d'une opération ($P=0.1$)
    - Valeurs numériques: $plus.minus$ valeur dans une plage de 1% de la valeur originale
    - _Byte Arrays_: Itérer et remplacer les _bytes_ ($P=0.01$)
    - _Byte_: $plus.minus v in [0,4]$ et clamping entre 0 et 255
]

#slide[
  #show: align.with(center + horizon)
  #v(-1.5em)
  #figure(image("assets/genetic_glitch_2.png", width: 75%))
  #figure(image("assets/genetic_glitch_3.png", width: 75%))
]


#slide[
  = Taux de fatalité
  #show: text.with(size: 20pt)

  - Jeu de données de 100 images
  - Convertir vers formats supportés
  - Appliquer un _glitch_ aléatoire sur chaque image
  - Vérifier si l'image est toujours lisible
  - Répéter 10 fois par format
]

#slide[
  == Résultats
  #show: text.with(size: 20pt)
  #show: align.with(center + horizon)
  #table(
    columns: 8,
    inset: 6pt,
    [ ], [bmp], [gif], [jpeg], [png], [raw], [tiff], [],
    [`insert`], [0.000], [0.168], [0.007], [0.997], [0.009], [0.998], [0.363],
    [`delete`], [1.000], [0.166], [0.006], [1.000], [0.007], [1.000], [0.530],
    [`replace`], [0.018], [0.180], [0.120], [0.996], [0.101], [0.198], [0.269],
    [`and`], [0.000], [0.016], [0.002], [0.997], [0.000], [0.010], [0.171],
    [`xor`], [0.000], [0.024], [0.007], [0.998], [0.007], [0.014], [0.175],
    [`or`], [0.000], [0.013], [0.145], [0.999], [0.145], [0.149], [0.242],
    [`not`], [0.005], [0.610], [0.310], [1.000], [0.277], [0.650], [0.475],
    [`reverse`], [0.006], [0.124], [0.094], [1.000], [0.115], [0.436], [0.296],
    [ ], [0.129], [0.163], [0.086], [0.998], [0.083], [0.432], [],
  )

  _GlitchBot_ utilise `replace` seulement!
]

#slide[
  = Impact visuel
  #show: text.with(size: 20pt)

  - Jeu de données de 100 images
  - Appliquer un _glitch_ aléatoire sur chaque image
  - Convertir l'image source et _glitched_ en noire et blanc
  - Calculer la distance moyenne entre les images

  $
    d_"grayscale" (I_a, I_b) = (sum_(x=0)^(x<w)sum_(y=0)^(y<h) abs(I_a(x,y) - I_b(x,y))) / (w dot h)
  $
]


#slide[
  == Résultats
  #show: align.with(center + horizon)
  #show: text.with(size: 16pt)
  #table(
    columns: 8,
    inset: 6pt,
    [ ], [bmp], [gif], [jpeg], [png], [raw], [tiff], [],
    [`insert`],
    [0.0000096],
    [0.0014833],
    [0.0002334],
    [0.0000001],
    [0.0010734],
    [0.0000000],
    [0.0004666],

    [`delete`],
    [0.0000000],
    [0.0014758],
    [0.0002661],
    [0.0000000],
    [0.0012873],
    [0.0000000],
    [0.0005049],

    [`replace`],
    [0.0000155],
    [0.0204895],
    [0.0020033],
    [0.0000000],
    [0.0023181],
    [0.0007292],
    [0.0042593],

    [`and`],
    [0.0000000],
    [0.0006621],
    [0.0001791],
    [0.0000000],
    [0.0009455],
    [0.0000057],
    [0.0002987],

    [`xor`],
    [0.0000000],
    [0.0005464],
    [0.0002195],
    [0.0000000],
    [0.0010731],
    [0.0000012],
    [0.0003067],

    [`or`],
    [0.0000000],
    [0.0004249],
    [0.0001382],
    [0.0000000],
    [0.0009570],
    [0.0000094],
    [0.0002549],

    [`not`],
    [0.0000042],
    [0.0005932],
    [0.0001423],
    [0.0000000],
    [0.0007850],
    [0.0000061],
    [0.0002551],

    [`reverse`],
    [0.0000003],
    [0.0014581],
    [0.0001809],
    [0.0000000],
    [0.0010178],
    [0.0000098],
    [0.0004445],

    [ ],
    [0.0000037],
    [0.0033917],
    [0.0004203],
    [0.0000000],
    [0.0011821],
    [0.0000951],
    [],
  )
]


#slide[
  = _Unsupervised Evolutionary Art_
  #show: text.with(size: 20pt)

  Jeu de données de 500 images de peintures célèbres

  - 20 exécutions d’évolution non supervisée
  - Taille de la population: 100 (sélection aléatoire)
  - Sélection par tournoi: taille 2
  - 10 générations par exécution
  - Basée sur le Facteur de Contraste Global (GCF)
  - Évalue les différences de couleur/teinte au lieu du contraste d’intensité
]

#slide[
  == Résultats

  - Diversité visuelle atteinte, mais certaines similarités entre les exécutions
  - Aucune stratégie de diversité de population appliquée
  - Opération la plus fréquente: Remplacement (27%)
  - Les moins fréquentes: Suppression (7,8%), Insertion (7,1%)
  - Taux de mortalité: 13% - 20%
  - Un taux de létalité élevé agit probablement comme une pression de sélection négative
]

#slide[
  #show: align.with(center + horizon)
  #figure(
    image("assets/glitch_tournament.png", width: 100%),
  ) <fig-glitch_tournament>
]

#slide[
  == Futur

  - Approche multi-objectifs pour la métrique
  - Mesurer différence entre image source et _glitched_
]

#slide[
  #show: align.with(center + horizon)
  #heading(level: 1, outlined: false)[Le _Glitch Art_ génétique]
]

#slide[ ]

#slide[
  // #nocite-block[
  //   @bedardLocalAccountBell2024
  // ]
  #bibliography("./refs.bib", style: "ieee")
]
