/* Agent2API · CodeArts 每日福利领取（签到中心「活动福利」卡那颗「去领取」）。

   ── 入口只有签到中心一处 ────────────────────────────────────
   账号页曾有一颗同款按钮，已随签到动作整体收敛到「签到中心」——
   领取状态（已领取徽章）也在那边标记，这里只是流程本体，不再被账号页引用。

   ── 与 ZCode 那颗「领套餐」是两件事，所以另开一个文件 ─────────
   判据位不同（`welfare` vs `claim`）、流程不同（本家**不要**验证码）、
   后端端点也不同。共用一个位会让两边的按钮判据互相污染。

   ── 两步：先只读探测，再由用户明确点一下才发写请求 ────────────
     ① 探测  POST /api/accounts/{id}/codearts-welfare/preview  —— 动作是只读的，一个写请求都不发
     ② 领取  POST /api/accounts/{id}/codearts-welfare          —— 真的去领

   为什么值得先探测：领取是**外部服务的写操作**（点一下账号当天的领取机会就少一次），
   而「今天有什么可领、已经试过几次」这条信息是免费的。先看一眼再动手，
   也让最常见的两种结果（暂无可领 / 今天已经到账）不消耗一次点击。

   ── 后端返回的四种结果都不是错误 ─────────────────────────────
   `官方已确认到账` / `已领取并确认` / `暂无可领取活动` / `等待重试` ——
   只有 HTTP 非 2xx 才当失败处理。特别是第三种：它说的是「这个账号不在活动范围内」，
   不是「网关坏了」，显示成红色会让用户去查一个不存在的问题。

   ── 到账的是什么 ────────────────────────────────────────────
   ops 福利领的是**套餐赠送积分**，**不增加福利模型的 token 池**（两个账户各记各的，
   见后端 providers::codearts::balance 的模块头）。文案照实写，别让人以为
   领完就能多跑福利模型。

   依赖：`window.__TAURI_INTERNALS__.invoke('api_request')`（与 add-provider-forms.js
   的 postAccount 同一个通用入口，不为这两个端点去桥里加具名方法）、
   `window.wbConfirm`、`window.wbApp.{toast, refresh}`。脚本顺序见 index.html。 */
(() => {
  const escapeHtml = value => String(value ?? '')
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

  const describeError = error => {
    if (error instanceof Error && error.message) return error.message;
    const text = String(error ?? '').trim();
    return text || wbI18n.t('Unknown error');
  };

  /** 管理端点的通用调用（POST /api/accounts 在桥里没有具名方法，这里同一口径）。 */
  async function manage(method, path, body) {
    const internals = window.__TAURI_INTERNALS__;
    if (!internals || typeof internals.invoke !== 'function') {
      throw new Error(wbI18n.t('Desktop runtime unavailable (Tauri not initialized)'));
    }
    return internals.invoke('api_request', { request: { method, path, body } });
  }

  /** 一条活动 → 给人看的一行。状态用中文，但保留上游原码（排障时要对得上）。 */
  function describeCampaign(item) {
    const amount = Number(item.benefitAmount) || 0;
    const state = item.confirmed
      ? wbI18n.t('Credited')
      : item.localClaimed
        ? wbI18n.t('Claimed, awaiting official confirmation')
        : item.claimable
          ? wbI18n.t('Claimable')
          : wbI18n.t('Not claimable ({status})', {
            status: escapeHtml(item.status || wbI18n.t('Unknown')),
          });
    return `<li><b>${amount} ${escapeHtml(item.benefitUnit || wbI18n.t('Points'))}</b> · ${state}</li>`;
  }

  function summaryHtml(account, preview) {
    const campaigns = preview?.campaigns || [];
    const attemptsLeft = Number(preview?.attemptsLeft ?? 0);
    const waiting = Number(preview?.nextAttemptInMs ?? 0);
    return wbI18n.t('Account: <b>{name}</b><br>', {
      name: escapeHtml(account?.name || account?.id || ''),
    })
      + wbI18n.t('Tried {attempts} times today (Beijing time {day}); {left} attempts left{next}<br>', {
        day: escapeHtml(preview?.day || ''),
        attempts: preview?.attempts ?? 0,
        left: attemptsLeft,
        next: waiting ? wbI18n.t(', earliest next attempt in {minutes} min', {
          minutes: Math.ceil(waiting / 60000),
        }) : '',
      })
      + `<ul class="codearts-welfare-campaigns">${campaigns.map(describeCampaign).join('') || wbI18n.t('<li>No campaigns can be claimed automatically</li>')}</ul>`
      + wbI18n.t('<p class="muted">What you receive is <b>plan gift points</b>, which do not add to the welfare model token pool; '
        + 'Claims are counted per account; after official confirmation this machine will not claim again.</p>');
  }

  async function start(account) {
    // 失败一律红色 toast；成功与「无事可做」走下面的中性提示
    const fail = message => window.wbApp?.toast?.(message, 'err');
    if (!account?.id) return;
    let preview;
    try {
      preview = await manage('POST', `/api/accounts/${encodeURIComponent(account.id)}/codearts-welfare/preview`, {});
    } catch (error) {
      fail(wbI18n.t('Probe failed: {message}', { message: describeError(error) }));
      return;
    }
    if (!preview?.eligible) {
      // 最常见的两种结果不值得让用户多点一次：直接说清楚就收工
      window.wbApp?.toast?.(
        preview?.campaigns?.length
          ? wbI18n.t('No campaigns to claim today (all already credited)')
          : wbI18n.t('This account has no campaigns that can be claimed automatically right now (only the daily login gift points are claimed automatically)'),
      );
      return;
    }
    const ok = await window.wbConfirm?.ask?.({
      title: wbI18n.t('Claim CodeArts daily welfare'),
      html: summaryHtml(account, preview),
      okText: wbI18n.t('Claim and await confirmation'),
    });
    if (!ok) return;
    let result;
    try {
      result = await manage('POST', `/api/accounts/${encodeURIComponent(account.id)}/codearts-welfare`, {});
    } catch (error) {
      fail(wbI18n.t('Claim failed: {message}', { message: describeError(error) }));
      return;
    }
    if (result?.confirmed) {
      window.wbApp?.toast?.(`✅ ${result.result || wbI18n.t('Officially confirmed as credited')}`);
    } else {
      // 「已领取并确认」与「等待重试」都是真话，不是失败：用中性提示
      window.wbApp?.toast?.(result?.result || wbI18n.t('Not confirmed as credited this time'));
    }
    await window.wbApp?.refresh?.();
  }

  window.wbCodeArtsWelfare = { start };
})();
