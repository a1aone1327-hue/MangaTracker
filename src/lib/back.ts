// Системная кнопка «назад» на Android должна закрывать шторку, а не приложение.
// Каждый открытый слой кладёт запись в историю браузера; popstate закрывает верхний слой.

type Handler = () => void;

const stack: Handler[] = [];
let ignore = 0; // сколько ближайших popstate вызваны нами самими и их надо пропустить
let installed = false;

function install() {
  if (installed) return;
  installed = true;
  window.addEventListener('popstate', () => {
    if (ignore > 0) {
      ignore -= 1;
      return;
    }
    stack.pop()?.();
  });
}

/**
 * Регистрирует слой. Возвращает функцию, которую надо вызвать, когда слой закрылся
 * сам (не кнопкой «назад»): она уберёт лишнюю запись из истории.
 */
export function pushBack(handler: Handler): () => void {
  install();
  stack.push(handler);
  history.pushState({ layer: stack.length }, '');
  return () => {
    const i = stack.indexOf(handler);
    if (i === -1) return; // уже закрыт кнопкой «назад»
    stack.splice(i, 1);
    ignore += 1;
    history.back();
  };
}

/** Закрыть верхний слой так же, как это делает кнопка «назад». */
export function closeTop() {
  history.back();
}
