# Automação de sincronização, tags e releases do fork `rech-rustdesk`

> Documento de planejamento. **Nenhum código ou workflow foi alterado.** Objetivo: alinhar o entendimento antes de implementar.

- Fork: `RechInformatica/rech-rustdesk`
- Upstream: `rustdesk/rustdesk`
- Licença do projeto: **GNU AGPL-3.0** (arquivo [`LICENCE`](../LICENCE))
- Versão atual declarada: `1.4.8` ([`Cargo.toml:3`](../Cargo.toml))

---

## 1. O que você pediu, resumido

1. Sincronizar automaticamente o fork com o upstream (`rustdesk/rustdesk`).
2. Cada versão sincronizada virar uma **tag** no seu repositório.
3. Cada tag gerar um **artefato baixável** direto no GitHub (Release).
4. Poder **escolher quais versões liberar** para download — sua distribuição pode ficar propositalmente **algumas versões atrás** do upstream (defasada).
5. Confirmar se a **licença permite** pegar o RustDesk, colocar sua marca e distribuir.
6. Ver **quem mais faz isso** como referência.

Abaixo, cada ponto em detalhe.

---

## 1.1 Decisões já confirmadas

Estas já foram decididas e o restante do documento assume elas como fixas:

| Item | Decisão |
|---|---|
| Nome final do produto/marca | **Rech Rustdesk** |
| Convenção de tag | **`X.Y.Z-rech.N`** (ex.: `1.4.9-rech.1`), onde `X.Y.Z` é a versão do upstream usada como base e `N` incrementa a cada rebuild/release sua sobre essa mesma base |
| Branch de integração | **Sim, branch intermediária** entre o sync do upstream e o `master` do fork (chamada de `upstream-tracking` neste documento — nome definitivo a confirmar) |
| Plataformas a buildar | **Manter a matriz completa** — tudo que o projeto oficial builda hoje (Windows Flutter+Sciter, macOS, iOS, Android, Linux em 4 formatos, Web), sem cortes. Acompanha automaticamente o que o upstream buildar em cada versão sincronizada. |
| Mecanismo de "quais versões ficam disponíveis para download" | **Arquivo versionado no repositório** (ex.: `RELEASED_VERSIONS.md` ou `releases.json`), editado manualmente, que lista quais tags/releases já publicadas são consideradas "oficialmente liberadas". Tag e Release no GitHub são criadas normalmente para cada versão sincronizada; a defasagem proposital vem de você simplesmente não adicionar as versões mais novas a esse arquivo até decidir liberar. Detalhado em §10. |

Ainda em aberto (retomado na §9): assinatura de código própria.

---

## 2. Questão legal primeiro: a licença permite o que você quer fazer?

**Sim, permite — com obrigações específicas.** O RustDesk é licenciado sob **AGPL-3.0** (cliente e servidor). Pontos relevantes do texto em [`LICENCE`](../LICENCE):

| O que você quer fazer | Permitido pela AGPL-3.0? | Condição |
|---|---|---|
| Pegar o código, modificar, criar sua própria build | ✅ Sim | — |
| Distribuir essa build (binário) para clientes/terceiros | ✅ Sim | Precisa oferecer o **código-fonte correspondente** a quem recebe o binário (seção 6 da licença) |
| Colocar sua marca/nome/ícone (rebrand) | ✅ Sim | Precisa **deixar claro que é uma versão modificada** e não afirmar ser o RustDesk oficial (seção 5a: "must carry prominent notices stating that you modified it") — cuidado com **trademark** "RustDesk", que é separado de copyright (ver §2.3) |
| Cobrar por suporte, instalação, customização | ✅ Sim | A AGPL permite cobrar (seção 4: "You may charge any price or no price...and you may offer support or warranty protection for a fee") |
| Manter seu fork/modificações **fechadas** e só distribuir o binário | ❌ Não | Precisa disponibilizar o código-fonte modificado a quem recebe o software (seção 6) |
| Rodar uma versão modificada de um **servidor** (ex.: `hbb_server`/relay) acessível pela rede, sem publicar o fonte | ❌ Não | É exatamente o cenário que a **cláusula "Affero" (seção 13)** cobre: se usuários interagem remotamente com sua versão modificada via rede, você deve oferecer o código-fonte correspondente a esses usuários — diferente da GPL comum, aqui **não precisa nem distribuir binário** para o gatilho valer |
| Ficar "algumas versões atrasado" do upstream (não publicar toda mudança imediatamente) | ✅ Sim | AGPL não obriga a acompanhar o upstream nem a publicar builds automaticamente — só obriga que, **para a versão que você efetivamente distribuir/expuser em rede**, o fonte correspondente esteja disponível |
| Remover avisos de copyright/licença originais | ❌ Não | Seção 4/5b exigem manter avisos de licença e copyright |

### 2.1 Resumo prático

- Você **pode** criar "Rech RustDesk", com seu logo, seu nome, seus binários — é exatamente o que este fork já começou a fazer (veja commit `fix(custom-client)` no histórico).
- Sua obrigação central é **disponibilidade do código-fonte**: como este fork já é público no GitHub, isso já está satisfeito — desde que o fonte publicado corresponda de fato ao binário distribuído (por isso tag ↔ artefato ↔ commit precisam estar sempre coerentes, o que é justamente o que a automação abaixo garante).
- Se algum dia você rodar **servidor próprio** (`hbb_server`, `rendezvous_server` de `libs/hbb_common` ou fork do [`rustdesk-server`](https://github.com/rustdesk/rustdesk-server)) com modificações, a cláusula Affero (seção 13) se aplica separadamente: os usuários que se conectam a esse servidor têm direito ao fonte daquela versão do servidor.
- **Marca "RustDesk"**: a licença de copyright não te dá automaticamente direito de uso da marca registrada. Prática comum (ver §4): renomear o app/produto (ex.: "Rech Remote", "Rech RustDesk"), manter créditos ao projeto original no README/sobre, e não usar o logo oficial do RustDesk como se fosse produto deles. Isso é trademark, não copyright — se quiser certeza jurídica total, vale uma checagem pontual com jurídico, mas não é bloqueador para o que você está planejando.

### 2.2 Aviso de modificação recomendado (seção 5a da AGPL)

Ao publicar seu fork, é seguro (e obrigatório) deixar explícito algo como:

> "Este é um fork modificado do RustDesk (https://github.com/rustdesk/rustdesk), mantido pela Rech Informática desde 2026-08. Alterações: [changelog]. Distribuído sob os mesmos termos da GNU AGPLv3."

Isso já está implícito no seu `README.md`/`AGENTS.md` atuais mencionarem o fork, mas vale reforçar num aviso de licença explícito quando o "rebrand" avançar (ex.: nome do app, ícone).

### 2.3 Quem mais faz isso — outros forks/distribuidores comerciais do RustDesk

Isso é um padrão bem estabelecido no ecossistema RustDesk — vários exemplos públicos, todos operando sob a mesma lógica (AGPL + rebrand + build própria):

| Projeto | O que fazem | Referência |
|---|---|---|
| **RustDesk Server Pro / RustDesk próprio** | A própria RustDesk Inc. vende versão "Pro" do servidor e clientes com marca própria, builds próprias, em cima do mesmo código AGPL | https://rustdesk.com/ |
| **Forks corporativos com rebrand** (padrão comum no GitHub) | Empresas de suporte remoto/MSP pegam o fork, trocam logo/nome/servidor padrão (`rendezvous-server`, `key`) embutido no cliente, publicam via GitHub Releases próprios | busca `"custom-rendezvous-server" rustdesk fork` no GitHub mostra dezenas de forks fazendo exatamente isso |
| **Forks com build automatizada via GitHub Actions replicando o pipeline oficial** | Muitos forks simplesmente reaproveitam os workflows `.github/workflows/flutter-build.yml` do próprio RustDesk (como este repositório já tem, herdado do upstream) e trocam só variáveis de branding/servidor | padrão visível inspecionando forks populares listados em github.com/rustdesk/rustdesk/forks |

O seu caso (fork + build própria + release defasada) é o modelo mais comum entre integradores/MSPs que usam RustDesk como base — não é incomum nem "zona cinzenta": é o uso pretendido da AGPL para esse tipo de produto.

---

## 3. Arquitetura da automação proposta

Duas necessidades distintas, que devem ficar em **dois workflows separados**:

```
┌─────────────────────┐        ┌──────────────────────┐        ┌─────────────────────┐
│  Workflow A          │        │  Você decide          │        │  Workflow B          │
│  "Sync com upstream"  │ ───▶  │  quando promover       │ ───▶  │  "Tag & Release"      │
│  (roda sozinho, cron) │        │  (manual ou por regra) │        │  (build + artefato)  │
└─────────────────────┘        └──────────────────────┘        └─────────────────────┘
```

### 3.1 Workflow A — Sincronização automática com o upstream

**Objetivo:** manter uma branch (ex.: `upstream-sync` ou `master`, a decidir) atualizada com `rustdesk/rustdesk`, sem publicar nada automaticamente para os usuários finais.

- **Gatilho:** `schedule` (cron, ex.: 1x por dia) + `workflow_dispatch` (rodar manual quando quiser).
- **O que faz:**
  1. Adiciona/usa um remote `upstream` apontando para `https://github.com/rustdesk/rustdesk.git`.
  2. `git fetch upstream --tags`.
  3. Faz merge (ou rebase, a decidir) do que há de novo em `upstream/master` para a branch intermediária **`upstream-tracking`** — essa branch nunca é tocada manualmente, é só o "espelho + merges automáticos" do upstream.
  4. Abre um **Pull Request automático** de `upstream-tracking` para `master` do fork (não faz merge direto) — assim você revisa conflitos e decide o timing, especialmente porque o fork já tem customizações próprias (ex.: `fix(custom-client)` do histórico) que podem conflitar com mudanças do upstream. A branch intermediária também serve como "zona de quarentena": se um sync vier quebrado, ele fica isolado ali até alguém resolver, sem bloquear outros trabalhos em `master`.
  5. Opcional: roda o `ci.yml`/testes existentes nesse PR antes de você aprovar.
- **Ferramentas comuns para isso:** ação pronta como `github.com/marketplace/actions/pull-upstream-changes` ou script simples com `git fetch`/`git merge` + `peter-evans/create-pull-request`. Não precisa reinventar — é um padrão bem coberto por actions prontas.
- **Por que PR e não merge automático:** merges diretos automáticos são arriscados num fork com modificações próprias — conflitos de merge em código (não só em texto) podem quebrar build silenciosamente. Um PR te dá o ponto de controle.

### 3.2 Definição de versão/tag — decidido: `X.Y.Z-rech.N`

O upstream já versiona em `Cargo.toml` (hoje `1.4.8`) e já tem workflow (`flutter-tag.yml`) disparado por tags `v[0-9]+.[0-9]+.[0-9]+`. Convenção adotada: **espelhar a versão do upstream + sufixo `-rech.N`**.

Ex.: upstream lança `1.4.9` → quando você decidir liberar, tageia `1.4.9-rech.1`. Isso deixa claro qual base do upstream cada release sua usa, e o `N` permite reter builds internas (`-rech.1`, `-rech.2`, ...) para a **mesma** base do upstream, caso precise re-buildar por causa de um patch/branding seu sem mudar de versão upstream.

> Atenção técnica para a fase de implementação: a regex atual do `flutter-tag.yml` (`v[0-9]+.[0-9]+.[0-9]+-[0-9]+`) espera um sufixo **numérico** depois do hífen, não aceita `-rech.1` como está. Será preciso ajustar o `on.push.tags` desse workflow (ou criar um workflow de tag próprio do fork) para casar com `1.4.9-rech.1`.

### 3.3 Tag e Release são automáticas por versão sincronizada; a defasagem de download é controlada à parte

**Decisão (confirmada por você):** diferente da ideia inicial de "só cria tag quando eu mandar manualmente", o fluxo real fica mais simples:

1. Toda vez que um sync do upstream é aprovado e mergeado em `master` (fim do fluxo em §3.1), a automação **cria a tag normalmente** — `X.Y.Z-rech.1` seguindo a versão do upstream que acabou de entrar — sem precisar de um passo manual extra de "promover para release".
2. Toda tag criada dispara o **Workflow B** (§3.4), que builda e publica uma **GitHub Release** com os artefatos — isso já acontece "de graça" hoje, herdado do upstream.
3. Ou seja: **toda versão sincronizada vira tag + Release publicamente visível no GitHub**, cedo ou tarde. Isso por si só **não** significa que você está recomendando/divulgando aquela versão para seus clientes.
4. O controle de **"o que está disponível para download"** (o pedido original de poder "chavear as versões que vou liberar") passa a ser um mecanismo **separado e explícito**: um arquivo versionado no repositório, mantido manualmente — detalhado em §10.

Vantagem dessa separação: a defasagem deixa de depender de "lembrar de tagear" (que é fácil de esquecer ou atrasar por semanas) e passa a ser só "lembrar de atualizar um arquivo" quando você decidir promover uma versão já existente — a tag/Release em si nunca fica bloqueada esperando uma decisão sua, então o histórico de builds fica sempre completo e rastreável, e a curadoria de "o que é oficialmente recomendado" fica isolada num único lugar fácil de auditar (o próprio arquivo, com histórico de commits/PRs).

**Caveat técnico a registrar:** uma GitHub Release "normal" (não-draft) é **publicamente listável e baixável diretamente pela URL** assim que publicada — mesmo que o arquivo de versões liberadas ainda não a inclua. Isto é, o arquivo controla o que você **divulga oficialmente** (ex.: um link "Baixar Rech Rustdesk" no seu site ou README apontando só para a versão liberada), mas não impede tecnicamente alguém de achar uma tag mais nova direto na aba "Releases" do GitHub. Se no futuro isso for um problema (ex.: querer builds intermediárias 100% invisíveis até liberar), dá para publicar como `draft: true` e só "publicar" (via API/UI) quando o arquivo for atualizado — ficando anotado como possível evolução, não implementado agora.

### 3.4 Workflow B — Tag → Build → Release com artefato baixável

**Boa notícia:** o fork **já herdou do upstream** o workflow [`flutter-tag.yml`](../.github/workflows/flutter-tag.yml), que:

- dispara em `push` de tag no formato `v[0-9]+.[0-9]+.[0-9]+` (ou com sufixo `-N`);
- chama [`flutter-build.yml`](../.github/workflows/flutter-build.yml), que compila para Windows/macOS/Linux/Android etc.;
- usa `softprops/action-gh-release` para **publicar automaticamente uma GitHub Release** com os binários anexados como artefato baixável, marcada `prerelease: true` por padrão.

Ou seja: **grande parte do "Workflow B" já existe** neste repositório, herdada do fork original. O que falta ajustar (fase de implementação, não agora):

- Adaptar o **padrão de tag aceito** (`on.push.tags`) se você adotar sufixos tipo `-rech.1` (a regex atual `v[0-9]+.[0-9]+.[0-9]+-[0-9]+` já cobre um sufixo numérico simples, mas não `-rech.1`; teria que ajustar o regex ou usar `1.4.9-1` como convenção).
- Decidir `prerelease: true/false` conforme sua política de divulgação.
- Possivelmente reduzir a matriz de plataformas/arquiteturas se você não precisa compilar tudo que o upstream compila (economiza minutos de CI).
- Trocar segredos de assinatura de código (`SIGN_SECRET_KEY`, certificados macOS/Windows) pelos seus, se for assinar os binários com identidade da Rech.
- Ajustar branding (ícone, nome do app, servidor padrão embutido) antes do build — isso é código/config, fora do escopo deste documento.

---

## 4. Resumo do fluxo ponta a ponta proposto

```mermaid
flowchart LR
    A[rustdesk/rustdesk upstream] -- fetch/merge agendado --> B[Workflow A: Sync]
    B -- merge automático --> U[branch upstream-tracking]
    U -- abre PR --> C[Você revisa e aprova o merge]
    C -- merge no master do fork --> D[master do fork atualizado]
    D -- tag automática --> F[Workflow B: build + Release ex 1.4.9-rech.1]
    F -- build multi-plataforma --> G[GitHub Release publicada com artefatos]
    G -- você edita o arquivo de versões liberadas --> R[RELEASED_VERSIONS liberadas p/ download oficial]
    R --> H[Usuários baixam a versão que você liberou]
```

Pontos de controle humano (onde você decide, não a automação):
1. Aprovar o PR de sincronização (aceitar as mudanças do upstream) — único ponto onde uma mudança de código é revisada por alguém.
2. Editar o arquivo de versões liberadas para promover (ou não) uma tag/Release já publicada como "disponível para download oficial" — aqui mora a "defasagem proposital" (§3.3 e §10).
3. Decidir se/quando entra assinatura de código própria (§9), independente do fluxo de sync.

---

## 5. Item que ainda falta decidir

- **§9:** assinatura de código própria (Windows/macOS) — o que exige em certificados e segredos, e se você quer lançar já assinado ou adicionar isso depois.

As demais dúvidas do levantamento inicial já foram fechadas: CI explicado em detalhe (§7), plataformas mantidas 1:1 com o oficial (§8, decidido), e mecanismo de liberação de download via arquivo manual (§3.3 e §10, decidido).

---

## 6. Um fato que muda a conta de custo: **o repositório já é público**

Confirmado agora via API do GitHub (`api.github.com/repos/RechInformatica/rech-rustdesk` → `"private": false`, `"visibility": "public"`). Isso é importante porque, como visto em §7.3, **runners hospedados pela GitHub são grátis e sem limite de minutos em repositórios públicos** — muda a recomendação de "precisa de runner próprio" para "não precisa, a menos que você queira por outro motivo (velocidade, hardware específico, ou decidir tornar o repo privado no futuro)".

---

## 7. CI/CD no GitHub Actions, explicado a partir do que você já conhece do GitLab

Você já usa GitLab CI com **runner próprio, instalado nas máquinas dos devs**. A boa notícia: o GitHub Actions tem exatamente esse mesmo modelo disponível (self-hosted runner), **e também** oferece máquinas prontas da própria GitHub (hosted runner) — os dois convivem, inclusive no mesmo workflow, job por job.

### 7.1 Tradução de conceitos GitLab → GitHub

| GitLab CI/CD | GitHub Actions | Observação |
|---|---|---|
| `.gitlab-ci.yml` (um arquivo, `stages`) | `.github/workflows/*.yml` (vários arquivos, um por "pipeline") | Este fork já tem 11 arquivos em [`.github/workflows/`](../.github/workflows/) |
| `stages:` sequenciais | `jobs:` paralelos por padrão, com `needs:` para criar dependência | Sem `needs`, GitHub roda tudo ao mesmo tempo — inverso do GitLab, que é sequencial por padrão |
| GitLab Runner (agente instalado na máquina) | **GitHub Actions Runner** (agente instalado na máquina) | Praticamente o mesmo binário/conceito — self-hosted runner |
| `tags:` no job, runner registrado com tags | `runs-on: [self-hosted, minha-label]`, runner registrado com labels | Mesma lógica de casar job↔máquina por rótulo |
| Shared runners (SaaS da GitLab, minutos por plano) | GitHub-hosted runners (`ubuntu-latest`, `windows-latest`, `macos-latest`) | Equivalente direto — máquina efêmera provida pela plataforma |
| CI/CD Variables (Settings → CI/CD) | Secrets and variables → Actions (Settings do repo/org) | Mesmo propósito: segredos e variáveis injetados no job |
| Environment + "manual" job / proteção de branch | **Environments** com "required reviewers" | Para gate de aprovação humana antes de um deploy/release |
| Artifacts do job (`artifacts:` no `.gitlab-ci.yml`) | `actions/upload-artifact` (+ GitHub Releases para download público) | Este fork já usa isso em `flutter-build.yml` |
| Pipeline por push/tag/MR | `on: push / pull_request / schedule / workflow_dispatch` | `flutter-tag.yml` já dispara por tag, como visto em §3.4 |

### 7.2 Os dois tipos de "runner" no GitHub Actions

**a) GitHub-hosted runners (máquina da GitHub)**
- VM efêmera, criada do zero para cada job e destruída no final — nunca reaproveita estado entre jobs (bom para reprodutibilidade, ruim se você depende de cache local pesado sem usar `actions/cache`).
- Imagens prontas: `ubuntu-22.04`/`24.04`, `windows-2022`, `windows-11-arm` (preview), `macos-14`, `macos-15-intel`, `macos-latest` — **este fork já usa todas essas** (confirmei em [`flutter-build.yml`](../.github/workflows/flutter-build.yml)).
- Especificação padrão: 4 vCPU / 16GB RAM / 14GB SSD (Linux/Windows); macOS um pouco menor. Existem **"larger runners"** (mais CPU/RAM/disco, inclusive self-hosted-like mas gerenciados pela GitHub) — pagos, disponíveis em planos Team/Enterprise.

**b) Self-hosted runners (sua máquina — o modelo que você já usa no GitLab)**
- Você instala o agente (`actions-runner`) numa máquina sua (física, VM, ou até container), ele registra no repositório/organização e fica escutando por jobs.
- Registro: `Settings → Actions → Runners → New self-hosted runner` no repo (ou a nível de organização, para compartilhar entre repos) → baixa um script (`config.sh`/`config.cmd`) com um token temporário → roda como serviço.
- Suporta Linux, Windows, macOS, ARM — inclusive nas máquinas dos devs, exatamente como hoje no GitLab.
- Você rotula o runner (labels customizadas) e direciona jobs pra ele com `runs-on: [self-hosted, windows, gpu]` etc. — mesma lógica de `tags:` do GitLab Runner.
- **Não conta para cota de minutos paga do GitHub** — o "custo" é só a sua própria infraestrutura (o que você já paga hoje independente disso).

### 7.3 Quanto custa / o que é grátis

Isso muda bastante dependendo de o repositório ser **público** ou **privado** — e, como confirmado em §6, **o seu já é público**.

| Cenário | GitHub-hosted runner | Self-hosted runner |
|---|---|---|
| **Repositório público** (seu caso hoje) | ✅ **Minutos ilimitados e grátis**, para Linux, Windows e macOS, sujeitos só a limites de uso justo (job individual até 6h; workflow completo até 35 dias; jobs concorrentes: 20 no plano Free da organização, sendo até 5 macOS simultâneos) | Sempre grátis (custo é só sua infra) |
| **Repositório privado** | Cota mensal grátis por plano de conta/organização: Free = 2.000 min/mês, Pro = 3.000, Team = 3.000, Enterprise Cloud = 50.000. **Atenção ao multiplicador por SO**: Linux consome 1x, Windows 2x, macOS **10x** da cota — um job macOS de 20 min consome 200 min da cota. Passou da cota → cobra por minuto extra (ou trava, se não tiver forma de pagamento configurada) | Sempre grátis (não usa a cota de minutos hospedados) |
| Armazenamento de artifacts/log | Cota grátis: 500MB (Free) / 1GB (Pro) / 2GB (Team) / 50GB (Enterprise); acima disso cobra por GB | Não se aplica (artifacts continuam subindo para o GitHub mesmo com runner self-hosted, essa cota é sobre armazenamento no GitHub, não sobre o runner) |

**Conclusão direta para hoje:** como o repositório é público, a pipeline de build/release (`flutter-build.yml`, que já builda Windows + macOS + Linux + Android) **já roda de graça em runner da GitHub, sem precisar de runner próprio**, e sem limite de minutos preocupante. Isso é diferente da experiência que você tem hoje no GitLab (onde você precisa de runner próprio mesmo para uso básico).

### 7.4 Então quando vale a pena ter runner próprio no GitHub?

Mesmo sendo grátis via hosted runner, self-hosted ainda pode valer a pena por outros motivos (não custo):
- **Velocidade de iteração**: hosted runners entram em fila em horários de pico; máquina própria dedicada não enfileira.
- **Hardware específico**: se algum dia precisar de GPU para testes, ou mais RAM/CPU do que os 4vCPU/16GB padrão, sem pagar pelos "larger runners" da GitHub.
- **Cache pesado entre builds** (ex.: toolchains Rust/Flutter grandes) — self-hosted persistente mantém estado entre execuções, evitando rebaixar tudo do zero a cada job (hosted runner sempre começa do zero, mitigado parcialmente por `actions/cache`).
- **Se o repositório virar privado no futuro** (decisão de negócio) — aí a cota de minutos (e o multiplicador 10x do macOS) passa a doer de verdade, e reaproveitar as máquinas de dev que você já usa no GitLab volta a ser a opção mais econômica.

### 7.5 Um alerta de segurança importante para repositório público

**Não recomendo self-hosted runner neste repositório enquanto ele for público**, a menos que configurado com cuidado. Motivo: em repositório público, **qualquer pessoa pode abrir um Pull Request**, e por padrão um PR de um fork externo pode disparar um workflow que roda **na sua máquina** (self-hosted) — ou seja, execução de código arbitrário de estranhos na sua infraestrutura. A própria documentação da GitHub recomenda não usar self-hosted runners em repositórios públicos sem mitigação.

Mitigações, se algum dia quiser usar self-hosted mesmo assim:
- `Settings → Actions → General → Fork pull request workflows`: exigir aprovação manual ("Require approval for all outside collaborators" ou "for first-time contributors") antes de qualquer workflow de um PR externo rodar.
- Restringir quais workflows podem usar `runs-on: self-hosted` (não expor esse label em workflows que reagem a `pull_request` de forks).
- Usar runners **efêmeros** (se autodestroem após 1 job) para não deixar resíduo de execuções não confiáveis.

Como hoje o build oficial roda só por `push` de tag (ação sua, não de terceiros) e não por `pull_request`, o risco já é baixo — mas vale ter isso mapeado antes de decidir trazer máquina própria para dentro do GitHub Actions deste repo.

### 7.6 Recomendação prática

Para este projeto, **hoje**: não é necessário runner próprio. O pipeline de release pode rodar 100% em `windows-2022`/`macos-14`/`ubuntu-22.04` hospedados pela GitHub, de graça, como já está configurado nos workflows herdados do upstream. Revisitar essa decisão só se:
1. o repositório for para privado, ou
2. a fila de espera de runners hospedados virar um problema de fato (não é hoje, com uma pipeline por tag manual), ou
3. surgir uma necessidade de hardware que a GitHub não oferece hospedado.

---

## 8. Quais plataformas realmente buildar

O `flutter-build.yml` do upstream (herdado pelo fork) builda **14 jobs diferentes**, resumidos abaixo:

| Job | Plataforma | Publica artefato hoje? |
|---|---|---|
| `build-for-windows-flutter` | Windows x86_64 + arm64 (cliente atual, Flutter) | ✅ Sim |
| `build-for-windows-sciter` | Windows x86 (UI antiga, Sciter — legado, pré-Flutter) | ✅ Sim |
| `build-for-macOS` | macOS x86_64 + aarch64 (Flutter) | ✅ Sim |
| `build-rustdesk-ios` | iOS (Flutter) | ❌ Não (`softprops/action-gh-release` está comentado nesse job — só builda, não publica) |
| `build-rustdesk-android` / `build-rustdesk-android-universal` | Android (por ABI + universal) | ✅ Sim |
| `build-rustdesk-linux` | Linux `.deb` (Flutter) | ✅ Sim |
| `build-rustdesk-linux-sciter` | Linux (Sciter — legado) | ✅ Sim |
| `build-appimage` | Linux AppImage | ✅ Sim |
| `build-flatpak` | Linux Flatpak | ✅ Sim |
| `build-rustdesk-web` | Cliente Web (Flutter Web) | depende do job (não visto publicar release) |

**Decisão (confirmada por você): manter a matriz completa, 1:1 com o que o projeto oficial builda.** Nenhum job é cortado. Isso simplifica a manutenção do fork de outra forma: como a lista de jobs vem diretamente de cima do que o upstream builda a cada versão, o Workflow A (sync) naturalmente já traz qualquer plataforma nova que o upstream adicionar (ou remover) no futuro — não precisa de manutenção manual da matriz aqui.

Efeito prático dessa escolha, já mapeado em §7.3: como o repositório é público, isso **não gera custo** (minutos de runner hospedado são grátis independente de quantos jobs rodam); o único efeito é tempo total de execução do pipeline (mais jobs em paralelo = pipeline mais longa até todos terminarem, mas isso não bloqueia nada porque cada tag dispara sua própria execução independente).

---

## 9. Assinatura de código própria (Windows/macOS)

O workflow atual já tem estrutura pronta para assinatura, mas usando infraestrutura/segredos do RustDesk oficial — para uma release com a marca **Rech Rustdesk**, ela precisaria ser assinada com identidade própria. Dois mundos independentes:

### 9.1 Windows
- O `flutter-build.yml` chama um serviço de assinatura externo via `SIGN_BASE_URL` + `SIGN_SECRET_KEY` (visto em `res/job.py sign_files`) — é a infraestrutura de assinatura do RustDesk, não vai funcionar com segredo próprio sem entender esse serviço primeiro.
- Para assinar com certificado próprio, o caminho padrão é: comprar um **certificado de assinatura de código (Code Signing Certificate)**, hoje quase sempre em modalidade **EV (Extended Validation)** — sem EV, o Windows SmartScreen trata o instalador como "não confiável" até acumular reputação (pode levar semanas/meses de downloads para "esquentar"); com EV, a reputação é imediata.
- Custo de mercado: certificado EV gira em torno de **US$ 300–500/ano** (varia por emissora: DigiCert, Sectigo, SSL.com etc.), e normalmente exige um **token USB físico (HSM)** ou um HSM em nuvem (ex.: Azure Trusted Signing, mais barato e sem token físico — alternativa mais nova e recomendada pela própria Microsoft).
- Uso em CI: o certificado/chave vira um **GitHub Secret** (`Settings → Secrets and variables → Actions`), e o passo de assinatura no workflow troca a chamada ao serviço do RustDesk por `signtool` (ou Azure Trusted Signing action) usando esse segredo.

### 9.2 macOS
- O workflow já usa `rcodesign` (Apple Codesign, ferramenta open-source) para assinar e `codesign`/`notarytool` para notarização — a estrutura já existe, só precisa dos segredos certos.
- Precisa de conta **Apple Developer Program** (US$ 99/ano) para gerar um **Developer ID Application certificate**.
- Depois de assinado, precisa **notarizar** (enviar para a Apple validar automaticamente e receber um "ticket" — sem isso, o Gatekeeper do macOS bloqueia a abertura do app com aviso de "desenvolvedor não identificado").
- Segredos que entram no GitHub: certificado exportado (`.p12` + senha) e credenciais de API da Apple (`App Store Connect API Key`) para notarização automatizada — tudo como Secrets do repositório/organização.

### 9.3 Resumo de custo/esforço

| Item | Custo aproximado/ano | Esforço |
|---|---|---|
| Windows EV Code Signing (tradicional) | ~US$ 300–500 | Configurar HSM/token, trocar passo de assinatura no workflow |
| Windows via Azure Trusted Signing | Modelo de assinatura por uso, geralmente mais barato, sem token físico | Precisa de conta Azure + configurar a action oficial da Microsoft |
| Apple Developer Program | US$ 99 | Gerar certificado + configurar notarização (infra do workflow já existe) |

**Não é bloqueador para começar**: dá para ter release funcionando sem assinatura (usuário recebe aviso do SO, mas funciona), e adicionar assinatura depois como uma segunda fase. Ajuda a decidir se você quer lançar rápido sem assinatura primeiro, ou já nascer assinado.

---

## 10. Mecanismo de liberação de download — decidido: arquivo manual no repositório

Isso é a resposta direta ao pedido original #4 ("quero poder chavear as versões que vou liberar para download, deixando minha aplicação defasada do fork algumas versões"). Reforçando o que ficou definido em §3.3: **tag e Release acontecem automaticamente para toda versão sincronizada; o que fica "oficialmente disponível para download" é controlado à parte, por um arquivo versionado que você edita manualmente.**

### 10.1 Formato proposto do arquivo

Um arquivo simples na raiz (ou em `docs/`), por exemplo `RELEASED_VERSIONS.md` (texto, fácil de revisar em PR) ou `releases.json` (se algum dia quiser que um site/README leia isso programaticamente). Sugestão de conteúdo mínimo:

```json
{
  "latest_released": "1.4.7-rech.1",
  "released": [
    { "tag": "1.4.7-rech.1", "released_at": "2026-08-10", "notes": "Primeira liberação estável" },
    { "tag": "1.4.5-rech.2", "released_at": "2026-06-01", "notes": "Ainda suportada para downgrade" }
  ]
}
```

- `latest_released`: qual tag é "a atual" recomendada — o que qualquer link "Baixar" no seu site/README deveria apontar.
- `released`: histórico do que já foi liberado (útil se precisar manter uma versão anterior disponível para rollback/suporte, mesmo já tendo uma mais nova liberada).
- Tags que existem no GitHub mas **não** estão nesse arquivo = sincronizadas e buildadas, mas ainda não promovidas — é aí que mora a defasagem proposital.

### 10.2 Como isso se conecta com o resto da automação

- **Edição:** você (ou quem autorizar) edita esse arquivo diretamente via commit/PR no repositório — sem workflow especial disparando isso, é literalmente `git commit` numa branch + merge, como qualquer mudança de conteúdo.
- **Uso opcional, fase 2:** dá para ter um pequeno workflow que lê esse arquivo e atualiza automaticamente a seção "Download" do `README.md` (ou publica uma página simples) com o link para `latest_released` — assim o arquivo vira a única fonte de verdade, e você nunca esquece de atualizar um link solto em outro lugar. Não é necessário para o primeiro momento, mas é uma extensão natural.
- **Sem alerta automático de defasagem por enquanto** (conforme sua resposta) — você decide quando promover, sem lembrete automatizado. Vale registrar um ponto de atenção não-bloqueante: como RustDesk já teve CVEs de segurança corrigidos em versões passadas, uma defasagem grande entre "última tag sincronizada" e "última liberada no arquivo" pode um dia significar "estou represando uma correção de segurança conhecida". Não estou propondo nada automático para isso agora — só deixando registrado como algo para revisitar se a defasagem que você praticar na prática crescer muito.

---

## 11. Plano para colocar em produção

Sequência recomendada para sair deste documento e chegar a "primeira release oficial Rech Rustdesk disponível para download". Organizado em fases; cada fase só depende da anterior estar concluída.

> **Status em 2026-08-11: Fases 0 a 4 e 6 concluídas e testadas de ponta a ponta em produção** (tag real `1.4.9-rech.1`, Release publicada com todos os artefatos). Fase 5 (rebrand) é o próximo bloqueador antes de divulgar builds oficialmente. Detalhes de cada item abaixo.

### Fase 0 — Decisões e insumos que faltam antes de escrever qualquer workflow

- [x] Onde vivem os segredos: **nível de repositório** (`SYNC_PAT` como repository secret). Revisitar se algum dia houver mais de um repositório reaproveitando a mesma automação.
- [ ] Levantar **assets de marca**: ícone/logo do "Rech Rustdesk" em todos os formatos que o build espera (`.ico` Windows, `.icns` macOS, `.png` variados Android/Linux) — ainda pendente, é o principal insumo faltante pra Fase 5.
- [x] Quem além de você pode aprovar/liberar: decidido "só eu por enquanto".
- [x] Repositório continua público — confirmado via API (`"visibility": "public"`).

### Fase 1 — Configuração de base do repositório

- [x] `master` confirmado como branch padrão.
- [x] Branch `upstream-tracking` criada e no `origin`.
- [x] Branch protection em `master` ativa (confirmado via API pública: `"protected": true`, exige aprovação).
- [x] `Settings → Actions → General → Workflow permissions`: **Read and write** + **Allow GitHub Actions to create and approve pull requests** confirmados via API (`default_workflow_permissions: "write"`, `can_approve_pull_request_reviews: true`).
- [ ] `Fork pull request workflows` (exigir aprovação de colaboradores externos) — **ainda não confirmado/aplicado**. Baixo risco hoje (nada dispara via `pull_request` de fork externo), mas ficou pendente de verificação.

### Fase 2 — Workflow A: sincronização com o upstream

- [x] `.github/workflows/sync-upstream.yml` criado (`schedule` diário 03h UTC + `workflow_dispatch`).
- [x] Fetch/merge/push implementados na mão (sem action de terceiro) usando `gh` CLI para abrir o PR, em vez de `peter-evans/create-pull-request` como cogitado originalmente.
- [x] PR de sync já dispara `ci.yml`/`flutter-ci.yml` automaticamente (nenhum trabalho extra necessário - esses workflows já reagiam a `pull_request` de qualquer branch).
- [x] **Testado de ponta a ponta em produção**, não só isolado: rodou via `workflow_dispatch`, abriu PR real (#5), PR foi mesclado. Três bugs reais encontrados e corrigidos nesse processo (não previstos no plano original):
  1. `GITHUB_TOKEN` não pode alterar `.github/workflows/*` → precisou de PAT (`SYNC_PAT`, fine-grained, Contents+Workflows+Pull requests: write) no `checkout`.
  2. `gh pr create` com `GITHUB_TOKEN` falhava (`Resource not accessible by integration`) por política a nível de organização → passou a usar `SYNC_PAT` também nessa etapa.
  3. `gh pr create` com `SYNC_PAT` falhava (`Resource not accessible by personal access token`) porque fine-grained PAT não tem suporte completo à mutation GraphQL `createPullRequest` → trocado por chamada REST direta (`gh api repos/.../pulls`).
  - Também avaliamos a API nativa `merge-upstream` ("Sync fork" do GitHub) como alternativa: funciona, mas bate na mesma trava de permissão de workflows — mantido o fetch+merge+push manual por já estar validado.

### Fase 3 — Ajustar o Workflow B para o padrão de tag da Rech e para criação automática

- [x] Padrão `[0-9]+.[0-9]+.[0-9]+-rech.[0-9]+` adicionado ao `on.push.tags` do [`flutter-tag.yml`](../.github/workflows/flutter-tag.yml), sem remover os padrões originais do upstream.
- [x] `tag-release.yml` criado: dispara em `pull_request` (closed+merged, head=`upstream-tracking`) + `workflow_dispatch`, lê a versão do `Cargo.toml`, monta `X.Y.Z-rech.N` incrementando `N` se a base já existir.
- [x] `prerelease: true` mantido como está (herdado, sem alteração) — decisão implícita: o controle real de "disponível para download" é o `RELEASED_VERSIONS.md` (Fase 4/§10), não a flag de prerelease do GitHub.
- [x] **Testado de ponta a ponta em produção**: PR de sync mesclado → tag `1.4.9-rech.1` criada automaticamente → build completo disparado → Release publicada. Um quarto bug encontrado: a tag criada com `GITHUB_TOKEN` não disparava o `flutter-tag.yml` (pushes/tags via `GITHUB_TOKEN` não disparam outros workflows, proteção do GitHub contra recursão) → corrigido usando `SYNC_PAT` também no `tag-release.yml`.

### Fase 4 — Arquivo de versões liberadas

- [x] [`RELEASED_VERSIONS.md`](../RELEASED_VERSIONS.md) criado na raiz — ainda **vazio** (nenhuma versão promovida oficialmente ainda; `1.4.9-rech.1` existe e está buildada, mas não foi promovida).
- [x] Seção de download adicionada ao `README.md`, apontando para o `RELEASED_VERSIONS.md`.
- [x] Processo de liberação documentado — consolidado **dentro do próprio** `RELEASED_VERSIONS.md` (seção "Como liberar uma nova versão"), em vez de um arquivo `docs/COMO_LIBERAR_VERSAO.md` separado.

### Fase 5 — Rebrand mínimo viável (fora do escopo de workflow, mas bloqueador para produção)

**Próximo passo do cronograma - nada aqui foi feito ainda.** Não é o foco deste MD (que é sobre automação/CI), mas nenhuma release é "produção" com a marca RustDesk original ainda visível. Itens mínimos antes da primeira release pública:
- [ ] Trocar nome do produto exibido na UI (strings de app name).
- [ ] Trocar ícones/logo (Windows `.ico`, macOS `.icns`, Android/Linux `.png`).
- [ ] Adicionar o aviso de "versão modificada" exigido pela AGPL (§2.2) — sugestão: tela "Sobre" do app + `README.md`.
- [ ] Revisar se há URLs/servidores padrão apontando para infraestrutura do RustDesk oficial que deveriam apontar para a sua (ex.: servidor de rendezvous padrão, se aplicável ao seu caso de uso).
- [ ] Definir se o pipeline de assinatura de código (§9) entra nesta primeira release ou fica para depois — se ficar para depois, deixar isso explícito nas notas de release ("binário não assinado, aviso do SO é esperado").

### Fase 6 — Ensaio ponta a ponta (dry run) antes do primeiro lançamento real

- [x] Fluxo completo rodado de ponta a ponta com uma versão real do upstream (1.4.8 → 1.4.9): sync → PR → aprovação → merge → tag automática → build → Release. `RELEASED_VERSIONS.md` ainda não foi editado (nenhuma versão promovida por decisão - ver Fase 4).
- [ ] Baixar o artefato gerado e testar a instalação/execução numa máquina limpa — **ainda não feito** (só confirmamos que os links de download funcionam, não testamos a instalação de fato).
- [ ] Conferir se o aviso de licença/marca aparece corretamente no binário — **bloqueado pela Fase 5** (ainda não existe aviso pra conferir).
- [ ] Considerar a primeira release "de produção" (ex.: desmarcar `prerelease`) — decisão em aberto, depende da Fase 5 estar pronta primeiro.

### Fase 7 — Operação contínua (rotina pós-lançamento)

- [ ] Cadência sugerida: revisar PRs de sync semanalmente (nada te impede de revisar mais cedo se uma correção de segurança sair no upstream — vale acompanhar `github.com/rustdesk/rustdesk/security/advisories`).
- [ ] Sempre que decidir promover uma versão nova, seguir o processo documentado no `RELEASED_VERSIONS.md` (Fase 4).
- [ ] Reavaliar periodicamente (ex.: a cada trimestre) os itens que ficaram como "fase 2" neste documento: automação da seção Download no README, alerta de defasagem (§10.2), assinatura de código (§9), e se o repositório deveria continuar público.

---

## 12. Fontes usadas nesta análise

- [`LICENCE`](../LICENCE) deste repositório (texto integral da AGPL-3.0, seções 4, 5, 6 e 13 citadas acima).
- [`.github/workflows/flutter-tag.yml`](../.github/workflows/flutter-tag.yml) e [`.github/workflows/flutter-build.yml`](../.github/workflows/flutter-build.yml) já existentes no fork.
- Histórico de commits do fork (`fix(custom-client): show options, incoming-only`), indicando que a customização de marca/cliente já começou a existir na base de código.
- Consulta feita agora à API pública do GitHub (`GET /repos/RechInformatica/rech-rustdesk`) confirmando `"private": false` / `"visibility": "public"`.
