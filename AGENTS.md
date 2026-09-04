# AGENTS.md

Lire `CLAUDE.md` avant toute modification. Le contrat CLI et le schéma v2 sont
gelés : les préserver et exécuter les validations qui y sont indiquées.

## Site vitrine N2B

- Domaine canonique : `https://n2b.aphrody.com`.
- Le futur site doit être intégralement servi par Rust avec Axum et Tokio.
- Le service HTTP doit écouter uniquement sur `127.0.0.1:8086`; nginx termine
  TLS et constitue la seule entrée publique.
- Réutiliser la bibliothèque UI, les primitives de page, les en-têtes de
  sécurité et le pipeline média communs définis dans le dépôt Aphrody.
- Le site reste une vitrine technique minimale : documentation, téléchargement
  du CLI, état de service et liens publics. Aucun secret, chemin personnel,
  identifiant privé, télémétrie nominative ou contenu d'un dépôt privé.
- Ne jamais exposer directement un port de développement ou une API interne.
- Toute nouvelle crate porte `publish = false` jusqu'à une revue de publication.

La stratégie multi-sites et les contrats partagés sont documentés dans
`../aphrody/docs/SITES-PLATFORM.md` dans le workspace partagé.
