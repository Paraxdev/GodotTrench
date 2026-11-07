---
source: b890aefb21f6d8de673ab4ebc70a71d93fd598c5
---

# Alvos, parâmetros e inputs { #targets-parameters-and-inputs }

Uma conexão dá o nome do seu alvo, da sua entrada e de um parâmetro opcional como texto simples. Esta página mostra como esse texto
vira os nós, o método e os valores usados no jogo.

## Alvos { #targets }

O alvo de uma conexão, e de chaves como `call_target`, decide quais nós recebem a entrada. Na maioria das vezes é um targetname que
você deu a outra entidade. As outras formas alcançam grupos de coisas e nós fora do mapa.

| Alvo | Alcança | Use para |
| --- | --- | --- |
| `main_door` | Toda entidade com esse targetname no mesmo mapa | O caso normal, uma entidade nomeada ou várias que dividem um nome |
| `door_*` | Todo targetname que começa com `door_` | Abrir um conjunto inteiro de uma vez. Só um `*` no final funciona |
| `@enemies` | Todo nó do grupo do Godot `enemies`, em qualquer lugar da árvore | Coisas geradas em tempo de execução ou colocadas pelo código do jogo |
| `!player` | O primeiro nó do grupo `player` | Alcançar o jogador quando a cadeia perdeu o ativador |
| `!activator` | Quem iniciou a cadeia, `!caller` significa o mesmo | Ferir ou teletransportar quem tocou um gatilho |
| `!self` | A entidade dona da saída | Uma entidade agindo sobre si mesma, como um botão que se desativa |
| `/root/Game` | Um caminho de nó absoluto, precisa começar com `/` | Autoloads e outro código do jogo fora do mapa |

Os targetnames são procurados dentro do `FuncGodotMap` mais próximo, então dois mapas na mesma cena não alcançam os nomes um do
outro. Grupos, caminhos de nó e `!player` funcionam na árvore inteira, use-os quando um mapa precisa falar com outro.

> [!NOTE]
> No Source, `!caller` é a entidade que disparou a saída. Aqui ele significa o mesmo que `!activator`, use `!self` para a própria
> entidade.

### Mudanças em tempo de execução { #runtime-changes }

As buscas de targetname ficam em cache por mapa, por velocidade. Depois de gerar ou renomear entidades nomeadas em tempo de
execução, chame `TrenchDotIO.invalidate(node)` com qualquer nó desse mapa para que a próxima busca as veja. Renomear significa mudar
o meta `td_targetname`, o `name` do nó não conta. Destruir uma entidade não precisa de chamada, o cache percebe os nós liberados.

Os nós feitos por um spawner não têm targetname. Alcance-os pelo grupo deles, por exemplo `@enemies`.

## Parâmetros { #parameters }

O parâmetro é um valor que você digita na conexão e entrega à entrada, como `25` para `take_damage` ou uma velocidade para uma
porta. O texto vira o primeiro tipo em que couber:

| Você digita | A entrada recebe |
| --- | --- |
| `25` | O inteiro 25 |
| `0.5` | O float 0.5 |
| `true` ou `false` | Um bool |
| `0 64 0` | Um `Vector3`, de três números separados por espaços |
| `hello` | A string `"hello"`, quando nada mais serve |

Um array JSON como `[10, "$activator"]` se espalha em vários argumentos, para entradas que recebem mais de um. Os valores são
convertidos para os tipos que o método declara, então `1234` chega a um argumento `String` como texto. Quando o método recebe um nó
antes de qualquer outro argumento, um texto como `!player` ou um targetname é procurado e passado como esse nó.

Um parâmetro também pode ser um destes marcadores, preenchidos quando a entrada é chamada:

| Marcador | Vira |
| --- | --- |
| `$activator` | O ativador, o nó que iniciou a cadeia |
| `$self`, `$caller` | A própria entidade que recebe |
| `$position` | A posição global da entidade que recebe |
| `$caller_name` | O targetname da entidade que recebe, senão o nome do nó dela |

Os marcadores são preenchidos na entidade **que recebe**, não na que disparou. Eles precisam ser o parâmetro inteiro ou um elemento
inteiro do array, não são substituídos dentro de um texto mais longo.

## Sem parâmetro { #no-parameter }

Deixe o parâmetro vazio e o ativador vai para o primeiro argumento tipado como objeto, ou sem tipo e chamado `activator`, onde quer
que esteja. É por isso que `take_damage(amount, source)` o recebe em `source`. Os outros valores que a saída carregava preenchem os
argumentos restantes em ordem, então `changed(value)` ligado a `set_value` repassa o valor.

Isso repassa `false` também, então `switched(false)` ligado ao `add` de um contador soma 0. Defina um parâmetro quando quiser um
valor fixo.

## Como uma entrada é encontrada { #how-an-input-is-found }

Quando uma conexão dispara, o nome da entrada é comparado com o alvo nesta ordem:

1. Um método com esse nome, ou a sua grafia em PascalCase ou camelCase, então um `AddScore` em C# responde a `add_score`.
2. Uma propriedade com esse nome, definida com o parâmetro ou com o valor que a saída carregava. A entrada `speed` com parâmetro
    `4` muda a velocidade de uma porta, sem precisar de método extra.
3. Uma das entradas embutidas abaixo, que todo nó entende.

| Entrada | Efeito |
| --- | --- |
| `kill` | Libera o nó, removendo-o do jogo |
| `show`, `enable` | Torna-o visível e o deixa processar de novo |
| `hide`, `disable` | Torna-o invisível e pausa o processamento dele |
| `toggle` | Inverte só a visibilidade, o processamento fica como está |

Os métodos da própria entidade vencem, então `disable` em um gatilho desliga o gatilho em vez de escondê-lo. `show` e `hide` são os
embutidos, a menos que o script da entidade defina os seus. Qualquer outra entrada imprime
`[TD I/O] <node> has no input '<name>'`.
