# Versões liberadas para download — Rech Rustdesk

Fonte única de verdade sobre quais builds já publicadas (tags/Releases no GitHub)
são oficialmente recomendadas para download pelos clientes.

Toda versão sincronizada do upstream vira tag e Release automaticamente (ver
[`docs/AUTOMACAO-SYNC-RELEASE.md`](docs/AUTOMACAO-SYNC-RELEASE.md), seção 3.3) —
isso, por si só, **não** significa que ela está liberada. Uma versão só é
"oficial" quando aparece na tabela abaixo. É assim que ficamos deliberadamente
algumas versões atrás do upstream quando fizer sentido (ver seção 10 do
documento acima).

## Versão atual recomendada

_(nenhuma ainda — aguardando a primeira liberação)_

## Histórico de versões liberadas

| Tag | Liberada em | Notas |
|---|---|---|
| _(vazio até a primeira liberação)_ | | |

## Como liberar uma nova versão

1. Confirme que a tag já existe e tem uma Release publicada com os artefatos
   (aba **Actions** → `Tag Release` / aba **Releases** do repositório).
2. Baixe e teste o artefato correspondente (pelo menos Windows, que é a
   plataforma mais usada pelos clientes) antes de liberar.
3. Adicione uma linha na tabela acima com a tag, a data (`AAAA-MM-DD`) e uma
   nota curta (ex.: "primeira liberação estável", "correção de X").
4. Atualize "Versão atual recomendada" para essa tag, se for a mais nova
   liberada até agora.
5. Abra um Pull Request com essa mudança e peça aprovação, como qualquer
   outra mudança em `master` (`master` é protegida — exige aprovação).
6. Depois de mesclado, confirme se o link de download no
   [`README.md`](README.md) segue apontando para a versão certa.
