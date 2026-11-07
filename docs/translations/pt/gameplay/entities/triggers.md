---
source: c7fdbd8dcaa1ac8625c1d6e7014b686194c50f36
---

# Triggers { #triggers }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

Volumes brush invisíveis que reagem quando um corpo entra neles. É assim que um nível percebe o jogador: uma porta de entrada inicia
uma cutscene, um buraco fere, uma plataforma lança. Desenhe um com a ferramenta **Volume** (Shift+E).

## Inputs, outputs e keys compartilhados { #shared-inputs-outputs-and-keys }

Todo trigger abaixo tem estes além dos seus próprios.

* **Inputs:** `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Keys:** `filter_group` player, `start_disabled` 0

`filter_group` limita o trigger aos corpos desse grupo do Godot, para que inimigos e caixas não disparem um trigger
do jogador. Deixe vazio para aceitar qualquer corpo.

| Output | Dispara |
| --- | --- |
| `entered` | A cada entrada aceita enquanto ativado |
| `triggered` | Em uma entrada aceita, no máximo uma vez por `cooldown` para cada corpo. A maior parte das ligações usa este |
| `exited` | Na saída, mesmo com o trigger desativado, para desfazer algo quando o corpo sai |

Os triggers detectam corpos na camada de física 1. Se o jogador está em outra camada, adicione-a a **Trigger Collision Mask** nas
configurações do mapa (o recurso `FuncGodotMapSettings` no Godot), ou os triggers nunca disparam.

## trigger_once { #trigger_once }

Dispara uma vez e se desativa, para eventos únicos como uma cutscene ou uma dica. `enable` o arma de novo.

* **Keys:** `once` 1

## trigger_multiple { #trigger_multiple }

Dispara a cada entrada, no máximo uma vez por `cooldown` para cada corpo, para coisas que acontecem toda vez, como o sino de uma loja.

* **Keys:** `cooldown` 0.5

## trigger_hurt { #trigger_hurt }

Causa dano aos corpos enquanto ficam dentro, para lava, espinhos ou uma zona de morte abaixo do mapa. Ele chama
`damage_method(amount, source)` neles a cada `interval` segundos, com `damage * interval` como quantidade.

* **Outputs:** `hurt(activator)`, por corpo ferido por tick
* **Keys:** `damage` 10 (por segundo), `damage_method` take_damage, `interval` 0.5, `filter_group` vazio

Com o `filter_group` padrão vazio ele fere qualquer coisa que tenha o método, não só o jogador. Com um grupo definido,
`hurt` dispara para todo corpo do grupo, mesmo um sem o método.

## trigger_teleport { #trigger_teleport }

Move os corpos para a entidade nomeada em `destination`, em geral um
[`info_teleport_destination`](actors.md#info_teleport_destination), e os vira para o yaw dela. Ele os para, a menos que
`keep_velocity` seja 1, para um portal que o jogador deve atravessar caindo ou correndo sem perder velocidade.

* **Outputs:** `teleported(activator)`
* **Keys:** `destination`, `keep_velocity` 0, `cooldown` 0.5

## trigger_push { #trigger_push }

Empurra corpos com `push`, uma velocidade em unidades do mapa por segundo que você pode apontar com a alça dele na viewport. Com `once` 1 cada
entrada recebe um impulso, como uma plataforma de pulo. Com 0 ele empurra de forma constante como um túnel de vento ou uma esteira, e um corpo rígido
para de acelerar na velocidade `push`.

* **Outputs:** `pushed(activator)`, uma vez por entrada
* **Keys:** `push` 0 384 0, `filter_group` vazio, `once` 1

## trigger_call { #trigger_call }

Chama um método quando disparado, por padrão em `!activator`, então um trigger pode chamar direto o script do jogador, por
exemplo para salvar um checkpoint. `call_target` e `arguments` funcionam como em [`logic_call`](scripting.md#logic_call).

* **Inputs:** `trigger(activator)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area { #trigger_spawn_area }

Gera cenas em pontos aleatórios dentro do seu volume, por padrão quando o jogador entra, para uma emboscada quando o jogador entra
em uma sala. As keys funcionam como em [`info_spawner`](actors.md#info_spawner). `start` roda um timer de spawn quando `interval` é
maior que 0, e `toggle` aqui liga e desliga esse timer em vez do trigger.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (uma vez, na primeira vez que `total` é alcançado)
* **Keys:** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `spawn_group` enemies,
  `snap_to_ground` 1, `cooldown` 0.5

## trigger_area { #trigger_area }

Um volume `Area3D` simples, sem script, para ligar os eventos de entrar e sair de uma sala direto a outras entidades. Ele
não tem nenhuma das keys compartilhadas acima e dispara para todo corpo, então filtre na entidade que ele chama quando isso importar.

* **Outputs:** `body_entered(body)`, `body_exited(body)`
* **Keys:** `targetname`
