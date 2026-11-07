---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# Entidades em C# { #c-entities }

Uma classe C# pode ser uma entidade posicionável sem uma definição `.tres`. O addon lê os seus arquivos de código: os signals viram
saídas, os membros exportados viram chaves e os métodos marcados viram entradas.

Rodar entidades em C# precisa da versão .NET do Godot. O editor as lista de qualquer jeito, mas o Godot só consegue construir uma
quando roda a versão .NET, o projeto C# está compilado e a classe tem `[GlobalClass]`. Do contrário, uma entidade brush em C# é
construída como um `Node3D` simples sem colisão e uma entidade ponto como um vazio, e o painel Output do Godot diz
`Node Class ... not found` com essas três coisas para conferir.

## 1. Adicione o auxiliar { #1-add-the-helper }

Abra o painel **Referência** (*Exibir > Painéis > Referência*) e aperte **Adicionar auxiliar C# ao projeto** no canto superior
esquerdo dele.

![O painel Referência com Adicionar auxiliar C# ao projeto sob o mouse, o tooltip diz Escreve TrenchDot.cs, a ponte que as entidades em C# usam](../assets/csharp/reference-csharp-helper.png)

Ele escreve `res://addons/func_godot/csharp/TrenchDot.cs`, que guarda os atributos. Sem ele, o código da sua entidade não compila.

## 2. Declare uma entidade { #2-declare-an-entity }

```csharp
[GlobalClass]
[TrenchDotEntity("npc_guard", Description = "Patrolling guard",
    Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level); // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                          // key "walk_speed"
    [TrenchDotInput] public void Alert(Node activator) { }                        // input "alert"
}
```

| Parte | Por que importa |
| --- | --- |
| `[GlobalClass]` | Obrigatório, a construção do mapa cria o nó pelo nome da classe |
| `Size` | A caixa que o editor desenha para a entidade, como mínimos e depois máximos em unidades do mapa |
| `Solid = true` | Faz uma entidade brush, como uma porta, em vez de uma entidade ponto |
| `[TrenchDotInput]` | Marca as entradas. Se nenhum método o tiver, todo método `void` público com nome em maiúscula é uma entrada |

Os nomes são snake_case no mapa e PascalCase em C#. Para ler as chaves você mesmo, implemente
`_func_godot_apply_properties(Dictionary props)` e use os leitores do auxiliar, como `TrenchDot.Float(props, "walk_speed", 3.5f)`.

> [!TIP]
> A aba *Classe C#* do painel Referência escreve um esqueleto com os signals, exports e entradas de qualquer entidade, um começo
> rápido para a sua. Ela deixa de fora a linha `[TrenchDotEntity]`, adicione-a você mesmo.

## Onde o addon procura { #where-the-addon-looks }

O editor e a construção do mapa varrem, cada um, pastas atrás de classes de entidade em C#, definidas em dois lugares:

| Usado por | Configuração |
| --- | --- |
| O editor (exportação da configuração do jogo) | *Csharp Source Dirs* no recurso de configuração do jogo |
| A construção do mapa | Configuração de projeto `trenchdot/csharp_entity_dirs` |

Os dois usam `res://` por padrão e pulam `addons`, `bin`, `obj` e pastas ocultas.

> [!NOTE]
> Uma classe C# não pode substituir uma entidade da biblioteca. Se o FGD já define o classname dela, o FGD vence.

## I/O a partir do C# { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

A aba *Uso a partir do C#* do painel Referência gera esse código para qualquer entidade.

<div class="td-mcp" markdown>

## MCP { #mcp }

Um agente ou script que controla o editor pelo [servidor MCP](../mcp.md) recebe o mesmo código gerado da ferramenta
`code_reference`.

</div>
