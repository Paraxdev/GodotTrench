---
source: 425ace012cd6d8410cbf64a89bb7dc31275f8980
---

# Depurando as conexões { #debugging-wiring }

A maioria dos erros de conexão falha em silêncio: a porta simplesmente continua fechada. Confira nesta ordem, da checagem mais
rápida à mais completa.

> [!TIP]
> Uma porta ou gatilho que nem sequer bloqueia o jogador foi construído sem a sua definição, o que não é um erro de conexão. Veja
> [Entidades que não fazem nada](../godot/entities-troubleshooting.md).

## 1. O painel Problemas { #1-the-issues-panel }

Abra a aba **Problemas**, ao lado de Outliner e Histórico, à esquerda. Ela marca saídas apontadas para um targetname que nenhuma
entidade tem, e entradas que a classe do alvo não tem. Um erro de digitação em um targetname é o motivo mais comum de uma porta
continuar fechada.

![O painel Problemas com um erro: a saída triggered tem como alvo a entidade desconhecida 'hall_lihgt' em trigger_once, com um botão Corrigir](../assets/debugging/issues-panel.png)

Clique em um problema para selecionar a entidade dele e enquadrá-la nas vistas, depois corrija o nome no Inspetor.

> [!WARNING]
> **Corrigir** em um alvo ausente não adivinha o nome certo, ele exclui a conexão. Para um erro de digitação, corrija o nome à mão.

## 2. O grafo lógico { #2-the-logic-graph }

No [Grafo lógico](logic-graph.md) um alvo que ninguém tem aparece como um nó vermelho com um fio vermelho tracejado, dizendo que
nenhuma entidade tem esse nome.

![O Grafo lógico: a saída triggered de trigger_once corre por um fio vermelho tracejado até um nó vermelho hall_lihgt, nenhuma entidade tem este nome](../assets/debugging/graph-missing-target.png)

**Simular** dispara uma saída no papel e destaca a cadeia inteira, veja [Simular](logic-graph.md#simulate).

## 3. F3 no jogo { #3-f3-in-game }

Adicione um nó `TrenchDotDebugOverlay` à sua cena no Godot, ou adicione-o como autoload. Rode o jogo e aperte F3: ele lista as
saídas conforme disparam e desenha todo volume `Area3D` com o seu targetname, então você vê se um gatilho nunca disparou ou disparou
em nada.

Alterne-o de novo depois de gerar gatilhos, os volumes são coletados quando ele liga. A tecla é a propriedade `toggle_key` do
overlay.

## 4. Registro em log { #4-logging }

Coloque uma entidade `logic_debug` e marque a caixa `trace_all` dela no Inspetor. Toda saída que dispara é então impressa na saída
do Godot.

Pelo código, conecte-se a `TrenchDotIO.events().fired`, que é emitido para toda saída, inclusive as cujo alvo não resolveu a nada.

```gdscript
TrenchDotIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## Como é cada erro { #what-each-mistake-looks-like }

| Erro | Sintoma | Correção |
| --- | --- | --- |
| O alvo não resolve a nada | `[TD I/O] <node>.<output>: target '<name>' matches no node` na primeira vez que dispara | Faça o alvo combinar exatamente com o targetname da entidade, maiúsculas e minúsculas inclusive |
| O alvo não tem essa entrada | `[TD I/O] <node> has no input '<name>'` na saída do Godot | Escolha uma entrada que a classe do alvo tenha, veja a [Referência de entidades](entities/README.md) |
| Saída que a entidade nunca emite | `[TD I/O] <node> has no signal '<output>'` uma vez, quando o mapa é construído | Escolha uma saída que a classe tenha, ou dispare-a pelo código com `TrenchDotIO.fire_output()` |
| Entidade gerada ou renomeada em tempo de execução | Não é achada pelo targetname | Chame `TrenchDotIO.invalidate()`, veja [Mudanças em tempo de execução](parameters.md#runtime-changes) |
