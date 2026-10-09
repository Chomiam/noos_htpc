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

## 🏷️ Règle n°2 : Montée de Version (v.X.Y+1) & Mise à Jour Obligatoire de `flake.lock`

1. **Incrémentation stricte de version (`v.X.Y+1`) :**
   * Chaque nouvelle release fait **obligatoirement** l'objet d'une montée en version incrémentale selon la formule `v.X.Y+1` (ex: `v0.1.0` ➔ `v0.2.0`, etc.).
   * La version doit être synchronisée de façon rigoureuse dans l'ensemble des fichiers du projet :
     * `dashboard/Cargo.toml` (`version = "X.Y.Z"`)
     * `dashboard/tauri.conf.json` (`"version": "X.Y.Z"`)
     * `installer/Cargo.toml` (`version = "X.Y.Z"`)
     * `installer/tauri.conf.json` (`"version": "X.Y.Z"`)
     * `flake.nix` (métadonnées et descriptions de paquets)

2. **Mise à jour impérative du fichier `flake.lock` dans le workflow de release :**
   * Le workflow de mise à jour comprend **obligatoirement** l'actualisation et la synchronisation du fichier `flake.lock` :
     ```bash
     nix flake update
     ```
   * Cette actualisation garantit que la configuration NixOS, les modules TV, le dashboard et les dépendances (nixpkgs, nix-flatpak) ciblent les révisions les plus récentes et sécurisées.
   * Le fichier `flake.lock` mis à jour doit être tracé et validé dans le commit de montée de version.

3. **Création du tag Git & Déploiement :**
   * Le tag Git correspondant (`git tag v.X.Y+1`) est créé lors de la release officielle.
   * Le système de mise à jour intégré au Dashboard TV permet à l'utilisateur de choisir entre le canal `stable` et le canal `testing`, de prévisualiser les commits/modifications et d'appliquer la mise à jour sans mot de passe sudo.

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
