---
source: 5a39bcf2beab2f567018f04963913fd06af5df16
---

# Overlays { #overlays }

Toda construção substitui o que o mapa gerou, então tudo o que você adiciona à mão sob o nó do mapa some no próximo salvamento.
Um overlay é o lugar do conteúdo do Godot que pertence ao mapa e precisa sobreviver a toda reconstrução: uma máquina de venda
com script próprio, um efeito de partículas ajustado no Godot, uma lâmpada com um AnimationPlayer.

![O pátio do distrito noturno: luzes de varal, vagalumes e um letreiro do overlay, lâmpadas do mapa](../assets/overlays/night-district-courtyard.jpg)

## Adicionando um { #adding-one }

1. Adicione o nó de overlay

    No Godot, adicione um nó `TrenchDotOverlay` como filho direto do `FuncGodotMap`.

2. Ponha seus nós embaixo dele

    Construções, modo ao vivo e hot reload nunca tocam nele, então os nós dele mantêm o estado e as conexões de sinal e são salvos
    com a sua cena.

(Para um único nó você pode pular o overlay: um filho direto do mapa no grupo `trenchdot_keep` também sobrevive às construções.
Um overlay em outro lugar da cena pode apontar para o mapa dele com *Map Path*, e então segue a
transformação do mapa.)

O conteúdo do overlay usa o espaço local do mapa em metros. A 32 unidades por metro, um filho em `(2, 0, -3)` fica onde
o TrenchDot mostra `(64, 0, -96)`.

## Um overlay na própria cena { #an-overlay-in-its-own-scene }

Um overlay também pode ser uma cena própria. Isso compensa quando várias cenas usam o mesmo mapa e devem compartilhar o
overlay, quando uma pessoa trabalha na cena do nível e outra no overlay, ou quando um script gera a cena do
nível, como na demo.

1. Faça uma cena cuja raiz é um `TrenchDotOverlay`.
2. Arraste o `.tscn` para o nó `FuncGodotMap`, para que seja instanciado como filho direto do mapa.

O Godot mostra uma cena instanciada como um nó só, sem seta para expandi-lo. Para chegar ao conteúdo dela:

| Você quer | Faça isto |
| --- | --- |
| Editar o overlay em todo lugar onde é usado | Clique no ícone *Open in Editor* ao lado do nó no dock Scene, ou dê um duplo clique no `.tscn` no dock FileSystem |
| Ver ou mudar os nós dele só nesta cena | Clique com o botão direito no nó e ligue *Editable Children* |

Aberta sozinha, a cena do overlay mostra um aviso na raiz porque não está sob um mapa. Isso é esperado. As
caixas tracejadas no TrenchDot se atualizam na próxima vez que a cena com o mapa é construída ou salva, já que só ali o
overlay conhece o mapa dele.

## Conectando overlays { #wiring-overlays }

Nós de overlay e entidades do mapa podem se disparar mutuamente pelas mesmas saídas e entradas que as entidades do mapa usam entre
si.

| Você quer | Faça isto |
| --- | --- |
| Que uma saída do mapa chegue a um nó de overlay | Dê ao nó um filho `TrenchDotOverlayIO` e defina o *Targetname* dele |
| Que um nó de overlay dispare em entidades do mapa | Adicione um filho `TrenchDotOutput`, escolha o sinal em *Output* e preencha alvo, entrada e parâmetro |
| Que o conteúdo ande junto com uma entidade em movimento | Ponha-o sob um `TrenchDotAnchor` com *Target* definido como o targetname da entidade |

Uma entrada enviada a um `TrenchDotOverlayIO` chama o método do nó com esse nome, define a propriedade dele com esse nome, ou
roda `show`, `hide` ou `toggle`. Então nós sem script também funcionam: a entrada `emitting` com o parâmetro `true` inicia um
`GPUParticles3D`. Toda entrada também emite `input_received(input, parameter, activator)`.

Um `TrenchDotOutput` escuta qualquer sinal do pai dele, então um `Area3D` dispara em `body_entered` sem código. Para
os seus próprios eventos, adicione um sinal ao script do pai e escolha-o:

```gdscript
extends StaticBody3D

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

Uma âncora mantém o deslocamento onde você a colocou e segue a entidade por reconstruções, edições ao vivo e movimento, então uma
placa de aviso sobe junto com o portão de enrolar.

> [!WARNING]
> As conexões sobrevivem às reconstruções, o estado não. As entidades do mapa voltam ao estado inicial, então conecte-se ao
> sinal `map_rebuilt(map)` do overlay e devolva o seu estado, por exemplo ligue as lâmpadas de novo.

## Veja na demo { #see-it-in-the-demo }

O distrito noturno do projeto de demo tem tudo isso. A cena dele `res://demo/showcase/night_district.scn` instancia a
cena de overlay `res://demo/overlays/night_district_overlay.tscn` sob o mapa, porque um script reconstrói as cenas de
showcase. Abra a cena do overlay e você vai achar uma caixa de disjuntores que liga as lâmpadas do pátio, luzes de varal que um
trigger do mapa acende, e uma placa andando junto com o portão de enrolar da garagem.
