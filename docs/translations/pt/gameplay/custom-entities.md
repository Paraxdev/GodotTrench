---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# Entidades próprias em GDScript { #custom-entities-in-gdscript }

Qualquer script pode ser uma entidade. Declare um **signal** para cada saída e um **método** para cada entrada, depois emita o
signal quando a coisa acontecer. Este alarme tem uma saída, `tripped`, e duas entradas, `trip` e `disarm`:

```gdscript
@tool
class_name Alarm extends Node3D

signal tripped(activator: Node)

@export var armed := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	armed = TrenchDotIO.to_bool(props.get("armed", armed))

func trip(activator: Node = null) -> void:
	if armed:
		tripped.emit(activator)

func disarm() -> void:
	armed = false
```

Faça do ativador o primeiro argumento `Node` dos seus signals, para que as cadeias mais adiante ainda saibam quem as iniciou.

As chaves do mapa chegam em `_func_godot_apply_properties`. Leia-as com `TrenchDotIO.to_bool`, `to_vector3` e `to_color`, que aceitam
tanto texto quanto valores já convertidos. Os nós que você adiciona ali são salvos com a cena. Os nós que você adiciona em `_ready`
não são, eles são feitos de novo toda vez que a cena carrega, o que serve ao que só o jogo em execução precisa. Em uma entidade com
um *Scene File*, o Godot salva os nós que vêm dessa cena como o arquivo da cena os tem, então as mudanças que você faz neles ali se
perdem. Os nós que você adiciona continuam sendo salvos, e as mudanças nos nós da própria cena pertencem a `_ready`.

O script precisa de `@tool`, porque os mapas são construídos no editor do Godot, onde só scripts tool rodam os seus métodos.

## Tornando-a posicionável { #making-it-placeable }

O editor e o Godot aprendem sobre as classes de entidade pelo FGD do seu projeto, a lista de definições de entidade. Cada definição
é um recurso próprio, e o tipo que você escolhe decide o que o Godot constrói:

| Sua entidade | Definição | O que definir |
| --- | --- | --- |
| Uma entidade ponto, em um lugar só, como uma lâmpada ou um item | `FuncGodotFGDPointClass` | *Script Class* para o seu script. Para gerar uma cena em vez disso, defina *Scene File*, e um *Script Class* então substitui o script da raiz da cena |
| Uma entidade brush, construída a partir de brushes, como uma porta ou um gatilho | `FuncGodotFGDSolidClass` | *Script Class* para o seu script, e *Node Class* para o corpo que ele estende: `StaticBody3D`, `AnimatableBody3D` ou `Area3D`. Se ficar vazio, o Godot constrói o corpo que o script estende, ou um `StaticBody3D` quando o script estende `Node3D` ou não há nenhum. Veja [Colisão](entities/collision.md) |

Esta é a definição do alarme acima, salva como `res://entities/alarm.tres` ao lado do seu script `alarm.gd`:

```
[gd_resource type="Resource" script_class="FuncGodotFGDPointClass" load_steps=3 format=3]

[ext_resource type="Script" path="res://addons/func_godot/src/fgd/func_godot_fgd_point_class.gd" id="1_class"]
[ext_resource type="Script" path="res://entities/alarm.gd" id="2_script"]

[resource]
script = ExtResource("1_class")
classname = "alarm"
description = "Trips when told to, until disarmed"
script_class = ExtResource("2_script")
class_properties = Dictionary[String, Variant]({
"targetname": "",
"armed": true
})
node_class = "Node3D"
```

Adicione-a a *Entity Definitions* do FGD que tanto as configurações do mapa quanto a configuração do jogo usam, veja
[Configuração do projeto](../godot/project-setup.md#your-own-resources). A configuração do jogo então a exporta para o editor, com
os seus signals como saídas e os seus métodos que não começam com `_` como entradas. Se uma entidade ainda for construída como um nó
simples, veja [Entidades que não fazem nada](../godot/entities-troubleshooting.md).

Para um começo rápido, abra o painel **Referência** e escolha uma classe parecida com a sua. Na aba *Classe GDScript*, **Criar no
projeto** escreve esse esqueleto em `res://entities/`.

![O painel Referência na aba Classe GDScript, Criar no projeto sob o mouse com o seu tooltip, Escreve o arquivo em res://entities/ a menos que ele exista](../assets/custom-entities/reference-create-in-project.png)

Na aba *FGD (.tres)*, **Criar no projeto** escreve uma definição e o script que ela executa, ambos com o nome da classe precedido de
`my_`, então `func_button` dá `res://entities/my_func_button.tres` e `my_func_button.gd`. Adicione o `.tres` a *Entity Definitions*
do seu FGD, e o Godot o constrói.

![O painel Referência na aba FGD (.tres), Criar no projeto sob o mouse com o seu tooltip, Escreve res://entities/my_func_button.tres, uma definição com um classname próprio, e o script que ela executa](../assets/custom-entities/reference-create-definition.png)

## Chamando o I/O pelo código { #calling-io-from-code }

O código do jogo pode usar a mesma máquina de uma conexão do mapa:

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

Prefira `invoke` a chamar o método direto, ele ajusta o parâmetro aos argumentos como uma conexão do mapa faz e trata as entradas
embutidas. `fire_output` dispara à mão toda conexão dessa saída, útil quando a entidade não tem um signal com esse nome.
