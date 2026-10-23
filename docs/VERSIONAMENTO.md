# Versionamento e publicação do Rech Rustdesk

Padrão adotado na RNS 573139-1 (projeto 4261). Substitui a convenção `X.Y.Z-rech.N` da seção 3.2 de [AUTOMACAO-SYNC-RELEASE.md](AUTOMACAO-SYNC-RELEASE.md).

## Formato

```
<versão do RustDesk>-r<N>        ex.: 1.5.1-r42
```

- **`1.5.1`**: versão do RustDesk de origem, lida do `version` do `Cargo.toml` no commit do build.
- **`r42`**: número do build Rech. É um **contador global**: cresce a cada versão Rech e **nunca volta a 1** quando a base do RustDesk muda.

```
1.5.1-r42  ->  1.5.1-r43  ->  (sync da 1.5.2 mesclado)  ->  1.5.2-r44
```

O `rN` sozinho identifica a entrega Rech (suporte: "qual build você tem?"). A parte `1.5.1` diz qual RustDesk está por baixo.

As tags antigas `X.Y.Z-rech.N` (até `1.5.1-rech.4`) ficam como histórico e não entram na contagem. A numeração nova começa em `r1`.

## Quando uma versão é gerada

Automaticamente, pelo [`tag-release.yml`](../.github/workflows/tag-release.yml), sem ninguém precisar lembrar de incrementar:

| Evento | Gera versão? |
|---|---|
| PR manual mesclado no `master` | **Sim**: `<base>-r<maior N + 1>`, no commit do merge |
| PR de sync (`upstream-tracking`) mesclado | Não. As mudanças do upstream entram na próxima versão Rech |
| PR com a label **`sem-release`** | Não (documentação, ajuste de workflow, ou para juntar vários PRs numa versão só) |
| `workflow_dispatch` do "Tag Release" | Sim, no `ref` informado (padrão `master`). Use para hotfix ou para fechar uma versão com PRs que entraram com `sem-release` |

Dois merges seguidos não pegam o mesmo número: o workflow roda um de cada vez (`concurrency`).

## O que a versão gera

1. **Tag** anotada `1.5.1-r42` ("Rech Rustdesk 1.5.1-r42").
2. **Build** ([`flutter-tag.yml`](../.github/workflows/flutter-tag.yml) e `flutter-build.yml`, só Windows, Linux e macOS, ver [WORKFLOWS-CORTES.md](WORKFLOWS-CORTES.md)) e **Release** marcada como prerelease.
3. Ao fim do build, o [`rech-finaliza-release.yml`](../.github/workflows/rech-finaliza-release.yml):
   - renomeia os arquivos: `rustdesk-1.5.1-x86_64.exe` passa a `rech-rustdesk-1.5.1-r42-x86_64.exe` (só o nome do download muda; o conteúdo do pacote é o do build);
   - grava o título **"Rech Rustdesk 1.5.1-r42"** e as notas: base do RustDesk, último sync do upstream incluído, commit do build e PRs desde a versão Rech anterior. Se o build falhou em algum job, as notas avisam.
4. Tela **"Sobre"** do app (`src/rech.rs`, `get_display_version`): `Rech Rustdesk 1.5.1-r42 (RustDesk 1.5.1)` no Windows e no macOS.

   Mostram só a versão do RustDesk:
   - builds sem tag Rech (local, CI de PR);
   - os pacotes Linux (deb, rpm, Arch, AppImage, flatpak). O binário Linux é compilado dentro de container (`run-on-arch-action`), que não recebe a variável `TAG_NAME`. Para incluir, bastaria repassar `TAG_NAME` no `env` dessa etapa do `flutter-build.yml`, mas isso aumenta o conflito de sync e ficou de fora.

## Liberação para clientes

Gerar a versão **não** a libera. Só vale o que está no [`RELEASED_VERSIONS.md`](../RELEASED_VERSIONS.md) (processo descrito lá).

## Por que não mexemos no `Cargo.toml`, no `pubspec.yaml` nem no `flutter-build.yml`

Esses arquivos vêm do upstream e mudam a cada sync. Gravar a versão Rech neles daria conflito em todo PR de sync. Por isso:

- a versão Rech existe só na **tag**;
- os nomes dos arquivos são trocados **depois** do build, pela API do GitHub, num workflow próprio do fork;
- o app lê a tag em tempo de compilação pela variável `TAG_NAME`, que o `flutter-build.yml` já exporta.

Consequência conferida: no **MSI (Windows)** dois builds Rech sobre a mesma base têm a mesma versão de produto, e instalam um por cima do outro porque o `Package.wxs` tem `AllowSameVersionUpgrades="yes"`.

## Comparação de versões

`1.5.1-r42` não é SemVer estrito (no SemVer, `-r42` seria pré-release e `r100` viria antes de `r42`). O git (`--sort=v:refname`) e o GitHub ordenam certo, e nada no fluxo compara versões por SemVer. Para ordenar entregas, use o `N`.

## Configuração necessária no GitHub

- Label **`sem-release`** criada no repositório.
- Secret **`SYNC_PAT`** com escrita no repositório (o push da tag com o `GITHUB_TOKEN` padrão não dispara o build).
- PR de sync mesclado com **merge commit** (já exigido em [WORKFLOWS-CORTES.md](WORKFLOWS-CORTES.md), seção 4): as notas da Release acham o último sync por esse merge.
- PR vindo de fork externo não recebe secrets, então o `tag-release.yml` falha nesse caso. Gere a versão pelo `workflow_dispatch`.
