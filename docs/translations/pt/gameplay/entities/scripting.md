---
source: 2bf109094dcc42ebb11cf6c736e210c0e4e79b02
---

# Scripts e chamadas { #scripts-and-calls }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

Entidades de ponto para o que a ligação simples não alcança: o estado do jogo, o código do seu próprio jogo, animações e descobrir por que uma
cadeia não dispara.

## logic_script { #logic_script }

Executa um trecho de GDScript quando disparado, para um pouco de lógica que nenhuma entidade embutida oferece, como ler a vida do jogador,
sem escrever uma entidade própria.

* **Inputs:** `run(activator)`, `run_with(parameter)`
* **Outputs:** `ran(result)`, com o valor de retorno do trecho
* **Keys:** `source` vazio, `expression` vazio (uma linha só, usada quando `source` está vazio)

O trecho enxerga `this` (a própria entidade), `activator` (quem iniciou a cadeia), `parameter` (o valor
recebido), `io` (os auxiliares do `TrenchDotIO`, por exemplo para achar entidades pelo nome) e `tree` (o `SceneTree`).
`run_with` sempre tem um `activator` nulo. Um erro de compilação é registrado como aviso quando o mapa carrega. Este trecho fere
o jogador:

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> [!WARNING]
> Indente as linhas aninhadas com tabs. Espaços não compilam e dão um erro de indentação mista.

> [!WARNING]
> O trecho é código que vem dentro do mapa. Ele roda sempre que o jogo roda, com os mesmos direitos dos scripts do próprio
> projeto, então trate um mapa baixado que contém código como qualquer programa que você baixa: ele pode fazer qualquer coisa no seu
> computador. O painel Problemas avisa de todo `logic_script` que contém código, de toda saída que pode entregar código a um e de
> todo prefab com qualquer um dos dois, leia isso antes de jogar o mapa.

## logic_call { #logic_call }

Chama um método em qualquer alvo com argumentos fixos, para alcançar o código do seu jogo, como somar pontos em um autoload `Game`.
Ao contrário de um output simples, ele avisa quando o alvo ou o método não existe, então os erros de digitação aparecem no log. Os métodos
em C# funcionam com seus nomes em PascalCase.

* **Inputs:** `trigger(activator)`, `call_with(parameter)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `/root/Game`, `method` vazio, `arguments` `[]`

`call_target` é um caminho de nó, um `@group`, um targetname, `!activator` ou `!self`. `arguments` é um array JSON, como
`["key_red", "$activator"]`, e aceita os placeholders de [Parâmetros](../parameters.md), resolvidos no próprio
`logic_call`. `call_with` põe o parâmetro dele onde está `"$parameter"`, ou o passa como único argumento.

## logic_animate { #logic_animate }

Toca animações no primeiro `AnimationPlayer` em `target` ou abaixo dele, para personagens e props que trazem as próprias
animações. Defina `target` como `!activator` para animar quem iniciou a cadeia.

* **Inputs:** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Outputs:** `finished(anim)`
* **Keys:** `target` `!self`, `animation` vazio (tocada quando `play` não tem nome), `speed` 1.0

## logic_debug { #logic_debug }

Imprime uma linha em `write` e mostra as linhas recentes como um rótulo flutuante nas construções de debug, para você ver no jogo se
uma cadeia chega até ele e com qual parâmetro. Com `trace_all` ligado ele registra todo output do jogo, um jeito rápido de acompanhar uma
cadeia inteira. Veja [Depurando as conexões](../debugging.md).

* **Inputs:** `write(parameter)`, `toggle` (o rótulo), `show`, `hide`
* **Outputs:** `printed(text)`
* **Keys:** `message` `{name} received {parameter} from {activator}` (`{time}` também funciona), `show_label` 1,
  `trace_all` 0
