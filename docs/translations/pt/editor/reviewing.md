---
source: 77812fdfe3757a8be2d8e1be3d7fbd73a1ff678a
---

# Revisando alterações { #reviewing-changes }

Mantenha seus mapas no git e você consegue ver o que mudou entre duas versões, do mesmo jeito que revisa código. Para as alterações da sessão atual, o [painel Histórico](organizing.md#history) lista cada edição e desfaz até qualquer uma delas.

## Mapas no git { #maps-in-git }

Os arquivos `.tdm` são binários, mas o git ainda consegue mostrar diffs legíveis. Ele pede ao editor para imprimir cada versão como JSON e compara essas saídas. Você configura isso uma vez:

1. Adicione esta linha ao `.gitattributes` do seu projeto:

    ```
    *.tdm binary diff=tdm
    ```

2. Diga ao git como imprimir um mapa, uma vez por clone:

    ```sh
    git config diff.tdm.textconv "trenchdot --dump"
    ```

3. (Opcional) Mantenha backups e salvamentos automáticos fora do repositório com estas linhas no `.gitignore`:

    ```
    *.tdm.bak
    *.tdm.autosave
    ```

O `git diff` e o `git log -p` agora devem mostrar as alterações do mapa como JSON, em vez de só dizer que os arquivos binários diferem.

> [!WARNING]
> O `trenchdot` precisa estar no seu `PATH`, ou o `git diff` falha nos mapas.

Para obter esse JSON como arquivo, `trenchdot --to-json map.tdm map.json` o escreve, e `trenchdot --to-tdm map.json
map.tdm` o transforma de volta em um mapa binário. Veja [Linha de comando](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

As ferramentas de revisão do MCP mostram o que um agente de IA fez no seu mapa.

### Edições feitas por um agente de IA { #edits-made-by-an-ai-agent }

Quando um agente edita seu mapa por [MCP](../mcp.md), cada chamada de ferramenta é um passo de desfazer chamado `MCP: ...` no [painel Histórico](organizing.md#history). Um `run_script` inteiro também é um passo só, como `MCP: Add crates, 6 steps`. Desfaça esse passo para tirar o trabalho do agente.

![O painel Histórico com MCP: Add crates, 6 steps no topo, acima de edições feitas à mão como Renomear, Agrupar e Esvaziar](../assets/reviewing/history-mcp-script.png)

| Para ver | Peça ao agente |
| --- | --- |
| O que mudou desde o último salvamento | `changes_since`, que lista os nós adicionados, removidos, movidos ou editados de outra forma |
| O que a última chamada ou script dele mudou | `changes_since` com `undo_steps: 1` |
| O que mudou desde um commit do git | `changes_since` com `file` apontando para essa versão, obtida com `git show HEAD:maps/level.tdm > old.tdm` |

> [!TIP]
> Peça ao agente para colocar um trabalho de vários passos em um só `run_script` com um `label`, assim o trabalho todo é desfeito em um passo, sob um nome que você reconhece.

</div>
