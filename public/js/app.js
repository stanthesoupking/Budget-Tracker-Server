function loadView(viewParts) {

    viewParts = viewParts.split(":");

    let viewName = viewParts[0];
    let newState = viewParts[1];

    $('.loading-spinner-container').removeClass('hidden');
    $('.loading-spinner-container').fadeIn(200, () => {
        $.get(`views/${viewName}.html`, success = (data) => {
            currentView = viewName;
            stateData = newState;
            $('.view-container').html(data);

            $('.loading-spinner-container').fadeOut(200, () => {
                $('.loading-spinner-container').addClass('hidden');
            });
        });
    });
}

function reloadView() {
    loadView(currentView)
}

function loadTemplate(templateName, callback) {
    let res = $.get({
        url: `views/templates/${templateName}.html`,
        async: false
    });

    return res.responseText;
}

function displayToast(toastName) {
    $.get(`views/toasts/${toastName}.html`, success = (data) => {
        let toast = $(data);
        $('.toast-container').append(toast);
        toast.toast('show')
    });
}

function getAccessTokenFromCookies() {
    let match = document.cookie.match(/access_token="(.*)"/);

    if ((match != null) && (match.length >= 2)) {
        return match[1];
    } else {
        return null;
    }
}

function setAccessTokenCookie(accessToken) {
    document.cookie = `access_token="${accessToken}"`;
}

function clearAccessTokenCookie() {
    document.cookie = 'access_token=; expires=Thu, 01 Jan 1970 00:00:01 GMT';
}

// --- API Adapter Functions ---

function getAccessToken(username, password) {
    return $.ajax("api/get_access_token", {
        data: JSON.stringify({
            username,
            password
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getBudgets(access_token) {
    return $.ajax("api/list/budgets", {
        data: JSON.stringify({
            access_token
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getBudget(access_token, budget_id) {
    return $.ajax("api/get/budget", {
        data: JSON.stringify({
            access_token,
            id: Number(budget_id)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function createBudget(access_token, budgetName, budgetSpendLimit, budgetPeriodLength) {
    return $.ajax("api/add/budget", {
        data: JSON.stringify({
            access_token,
            budget_name: budgetName,
            budget_spend_limit: Number(budgetSpendLimit),
            budget_period_length: Number(budgetPeriodLength)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}