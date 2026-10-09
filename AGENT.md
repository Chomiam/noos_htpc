# 📋 AGENT.md — Règles de Gouvernance et Développement : Noos HTPC

> **CONSIGNE IMPÉRATIVE POUR L'AGENT IA ET LES CONTRIBUTEURS :**
> Ce document définit les règles absolues, obligatoires et non-négociables régissant le développement, le versioning, l'architecture et les opérations Git sur le projet **Noos HTPC** (`/home/chomiam/Projets/noos_htpc/`).

---

## 🛡️ Règle n°1 : Travail EXCLUSIF sur la branche `testing`

1. **Interdiction formelle de commiter, modifier ou pousser directement sur la branche `stable` ou `main`.**
2. **Tout développement, correction de bug, refactoring ou ajout de fonctionnalité s'effectue UNIQUEMENT sur la branche `testing`.**
3. **Déploiement en `stable` sous autorisation expresse :**
   * Aucune fusion (merge), aucun push ni aucune release vers la branche `stable` ne doit être réalisé sans **l'accord formel et textuel explicite de l'utilisateur** (ex: *"Déploie en stable"*, *"Publie la version stable"*).
4. **Protocole de travail systématique :**
   * Vérifier systématiquement la branche courante avant toute modification : `git branch`.
   * Si la branche active n'est pas `testing`, basculer immédiatement : `git checkout testing`.

---

## 🏷️ Règle n°2 : Montée de Version & Mise à Jour Obligatoire de `flake.lock`

1. **Incrémentation stricte de version (SemVer) :**
   * Chaque nouvelle release officielle fait **obligatoirement** l'objet d'une montée en version (`vX.Y.Z`).
   * La version doit être synchronisée de façon cohérente dans l'ensemble des fichiers du projet :
     * `installer/Cargo.toml` (`version = "X.Y.Z"`)
     * `installer/tauri.conf.json` (`"version": "X.Y.Z"`)
     * `flake.nix` (métadonnées et descriptions de paquets)
2. **Mise à jour impérative du fichier `flake.lock` :**
   * Dans le workflow de release, il est **impératif et obligatoire** que le fichier `flake.lock` soit mis à jour et validé.
   * La mise à jour garantit que l'image ISO et la distribution ciblent les révisions exactes des paquets, évitant tout décalage d'arborescence ou dépendance fantôme :
     ```bash
     nix flake update
     # ou ciblé :
     nix flake lock --update-input nixpkgs
     ```
   * Le fichier `flake.lock` mis à jour doit être tracé et commité dans le même commit de release que la montée de version.
3. **Création du tag Git :**
   * Le tag Git correspondant (`git tag vX.Y.Z`) est créé uniquement lors de la release officielle validée par l'utilisateur.

---

## 🚫 Règle n°3 : Interdiction des Builds Lourds Locaux

1. **Ne jamais exécuter de compilation lourde locale :**
   * Interdiction de lancer la génération complète d'une image ISO (`nix build .#iso`) ou des compilations binaires complètes/lourdes sur la machine de développement locale.
2. **Délégation CI/CD distante :**
   * La compilation des images ISO et des binaires d'installation est déléguée aux serveurs distants via **GitHub Actions** (`.github/workflows/release.yml`) avec alimentation du cache binaire Cachix (`steveos.cachix.org`).
3. **Opérations locales autorisées :**
   * Vérifications syntaxiques ultra-légères (`node -c`, vérifications de schémas JSON, validations de structures Nix).

---

## 🦀 Règle n°4 : Architecture Rust (Installateur Tauri) & Sécurité

1. **Politique Zéro-Panic :**
   * Bannissement absolu des `unwrap()` et `expect()` dans les flux d'exécution et les commandes d'installation.
   * Utilisation systématique de `Result<T, E>`, `thiserror` et `anyhow` pour un traitement robuste des erreurs.
2. **Sécurité Système & Injection de Commandes :**
   * Interdiction stricte des sous-shells non contrôlés (`bash -c "chaine concaténée"`).
   * Toutes les commandes système (partitionnement, montage, formatage, gestion réseau) doivent être exécutées via des appels vectorisés stricts (`std::process::Command` ou `tokio::process::Command`).

---

## ✍️ Règle n°5 : Commits & Messages en Français

1. **Formatage des commits :**
   * Utilisation du standard Conventional Commits avec description détaillée **en français** :
     * `feat(scope): ...` pour les nouvelles fonctionnalités
     * `fix(scope): ...` pour les corrections d'anomalies
     * `chore(release): bump version vX.Y.Z et synchronisation flake.lock`
2. **Traçabilité :** Mentionner systématiquement les identifiants de tickets ou de correctifs si applicables.
