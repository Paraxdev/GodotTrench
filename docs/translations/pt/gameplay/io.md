---
source: 757483e9586cbd895b01c35597f84b4f54089ad0
---

# Como funciona o I/O de entidades { #how-entity-io-works }

O I/O de entidades é como um mapa faz as coisas acontecerem sem escrever código, ligado do jeito que o Hammer faz para o Source.

Toda entidade tem **saídas**, eventos que ela anuncia, como um botão sendo apertado, e **entradas**, coisas que você pode mandar
ela fazer, como uma porta abrindo. Uma conexão diz: quando esta saída disparar, chame aquela entrada em um alvo. A ligação fica
salva no mapa, então você liga um botão a uma porta no editor e a porta abre no jogo.

## Uma conexão { #a-connection }

Uma saída pode ter qualquer número de conexões. Cada uma tem estes campos:

| Campo | O que faz |
| --- | --- |
| Saída | O evento desta entidade que inicia a conexão, por exemplo `pressed` em um botão |
| Alvo | Quais entidades recebem a chamada, em geral um targetname, veja [Alvos](parameters.md#targets) |
| Entrada | O que os alvos são mandados fazer, por exemplo `open` em uma porta |
| Parâmetro | Um valor opcional entregue à entrada, como uma quantidade de dano, veja [Parâmetros](parameters.md#parameters) |
| Atraso | Segundos de espera antes de chamar a entrada, 0 chama na hora. Use para escalonar eventos, como luzes que acendem uma depois da outra |
| Vezes | Quantas vezes a conexão pode disparar. -1 significa toda vez, 1 faz um evento de uso único |

Você as edita no Inspetor, em **Saídas** da entidade selecionada, ou como fios no [Grafo lógico](logic-graph.md). Os dois mudam as
mesmas saídas. (O Inspetor só tem uma caixa **disparar uma vez** para *Vezes*, para qualquer outra contagem use a caixa de conexão
no Grafo lógico.)

![A seção Saídas do Inspetor de um botão: saída pressed, alvo front_door, entrada open, um parâmetro vazio, atraso 0,00 s e a caixa disparar uma vez](../assets/io/inspector-outputs.png)

No Godot uma saída é um sinal e uma entrada é um método: `func_button` tem `signal pressed(activator)` e `func_door` tem
`func open()`. Os seus próprios scripts participam do mesmo jeito, veja [Entidades próprias](custom-entities.md).

## Quando uma saída dispara { #when-an-output-fires }

Depois de *Atraso* segundos, a entrada é chamada em todo nó que o alvo encontrar. Um alvo que não encontra nada não faz nada. Para
um targetname ou caminho de nó isso quase sempre é um erro de digitação, então ele imprime um aviso na primeira vez que dispara.
Uma chamada com atraso é descartada se a sua entidade saiu da árvore nesse meio tempo, por exemplo porque foi destruída.

Sem atraso, a cadeia inteira roda dentro do `emit` do sinal, antes de o código que emitiu continuar. Isso importa quando o seu
script lê o estado logo depois de emitir, as portas e contadores mais adiante já reagiram.

## O ativador { #the-activator }

O ativador é quem iniciou a cadeia, em geral o jogador que apertou o botão ou entrou no gatilho. Mais adiante na cadeia,
`!activator` significa esse nó, então um gatilho pode ferir ou teletransportar exatamente quem o tocou.

Ele é tirado do primeiro argumento `Node` do sinal que disparou, então só sobrevive enquanto cada sinal da cadeia repassar um. Os
sinais embutidos se comportam assim:

| Sinais | O que acontece com o ativador |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Repassado sem mudança, esses sinais carregam o nó que os causou |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Substituído pelo nó entre parênteses, então mais adiante `!activator` é esse corner, nó gerado ou trem |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Descartado. Eles não carregam nenhum nó, então daqui em diante `!activator` não encontra nada |

> [!TIP]
> Precisa do jogador vários passos adiante em uma cadeia que perdeu o ativador? Use `!player` como alvo, ele não depende da
> cadeia.
