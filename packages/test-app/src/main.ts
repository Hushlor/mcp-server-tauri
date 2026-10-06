import { invoke } from '@tauri-apps/api/core';
import { message, ask, confirm, open, save } from '@tauri-apps/plugin-dialog';

let greetInputEl: HTMLInputElement | null,
    greetMsgEl: HTMLElement | null;

async function greet(): Promise<void> {
   if (greetMsgEl && greetInputEl) {
      greetMsgEl.textContent = await invoke('greet', {
         name: greetInputEl.value,
      });
   }
}

function showDialogResult(dialogResult: HTMLElement, text: string): void {
   if (dialogResult) {
      dialogResult.textContent = text;
   }
}

function watchHtmlFileInput(selector: string, label: string, resultElement: HTMLElement): void {
   const input = document.querySelector(selector) as HTMLInputElement | null;

   input?.addEventListener('change', () => {
      const names = Array.from(input.files ?? []).map((file) => { return file.name; });

      showDialogResult(resultElement, `${label} result: ${names.length ? names.join(', ') : 'Cancelled'}`);
      input.value = '';
   });
   input?.addEventListener('cancel', () => {
      showDialogResult(resultElement, `${label} result: Cancelled`);
   });
}

window.addEventListener('DOMContentLoaded', () => {
   greetInputEl = document.querySelector('#greet-input');
   greetMsgEl = document.querySelector('#greet-msg');
   document.querySelector('#greet-form')?.addEventListener('submit', (e) => {
      e.preventDefault();
      greet();
   });

   const dialogResult = document.querySelector('#dialog-result') as HTMLElement;

   const htmlFileResult = document.querySelector('#html-file-result') as HTMLElement;

   watchHtmlFileInput('#html-file-single', 'HTML single file', htmlFileResult);
   watchHtmlFileInput('#html-file-multiple', 'HTML multiple files', htmlFileResult);

   document.querySelector('#dialog-message')?.addEventListener('click', () => {
      message('This is a test message from the demo app.', { title: 'MCP Dialog Test', kind: 'info' })
         .then(() => {
            showDialogResult(dialogResult, 'Message dialog closed.');
         });
   });

   document.querySelector('#dialog-ask')?.addEventListener('click', () => {
      ask('Do you want to proceed with this action?', { title: 'MCP Dialog Test', kind: 'warning' })
         .then((answer) => {
            showDialogResult(dialogResult, `Ask result: ${answer ? 'Yes' : 'No'}`);
         });
   });

   document.querySelector('#dialog-confirm')?.addEventListener('click', () => {
      confirm('Are you sure you want to continue?', { title: 'MCP Dialog Test', kind: 'info' })
         .then((confirmed) => {
            showDialogResult(dialogResult, `Confirm result: ${confirmed ? 'Ok' : 'Cancel'}`);
         });
   });

   document.querySelector('#dialog-open')?.addEventListener('click', () => {
      open({ multiple: false, directory: false })
         .then((file) => {
            showDialogResult(dialogResult, `Open result: ${file ?? 'Cancelled'}`);
         });
   });

   document.querySelector('#dialog-open-multiple')?.addEventListener('click', () => {
      open({ multiple: true, directory: false })
         .then((files) => {
            const result = Array.isArray(files) ? files.join('|') : files;

            showDialogResult(dialogResult, `Multiple open result: ${result ?? 'Cancelled'}`);
         });
   });

   document.querySelector('#dialog-folder')?.addEventListener('click', () => {
      open({ multiple: false, directory: true })
         .then((folder) => {
            showDialogResult(dialogResult, `Folder result: ${folder ?? 'Cancelled'}`);
         });
   });

   document.querySelector('#dialog-save')?.addEventListener('click', () => {
      save({ defaultPath: 'untitled.txt' })
         .then((path) => {
            showDialogResult(dialogResult, `Save result: ${path ?? 'Cancelled'}`);
         });
   });
});
