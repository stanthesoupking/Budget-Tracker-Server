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

function gotoView(viewParts) {
    location.hash = '#' + viewParts;
} 

function reloadView() {
    loadView(location.hash.substr(1));
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

function getAccessToken(email, password) {
    return $.ajax("api/get_access_token", {
        data: JSON.stringify({
            email,
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

function createBudget(access_token, budgetName, budgetSpendLimit, budgetPeriodLength, budgetStartDate) {
    return $.ajax("api/add/budget", {
        data: JSON.stringify({
            access_token,
            budget_name: budgetName,
            budget_spend_limit: Number(budgetSpendLimit),
            budget_period_length: Number(budgetPeriodLength),
            budget_start_date: budgetStartDate
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function deleteBudget(access_token, budgetID) {
    return $.ajax("api/delete/budget", {
        data: JSON.stringify({
            access_token,
            id: Number(budgetID)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getSharedWith(access_token, budget_id) {
    return $.ajax("api/list/can_access_budget", {
        data: JSON.stringify({
            access_token,
            id: Number(budget_id)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function registerAccount(email, firstName, lastName, password) {
    return $.ajax("api/register_user", {
        data: JSON.stringify({
            email: email,
            first_name: firstName,
            last_name: lastName,
            password: password
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function addUserToBudget(access_token, email, budgetID) {
    return $.ajax("api/add/can_access_budget", {
        data: JSON.stringify({
            access_token,
            budget_id: Number(budgetID),
            email: email
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function removeUserFromBudget(access_token, email, budgetID) {
    return $.ajax("api/delete/can_access_budget", {
        data: JSON.stringify({
            access_token,
            budget_id: Number(budgetID),
            email: email
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function addTransactionToBudget(access_token, budgetID,
    transactionName, transactionDescription, transactionAmount,
    transactionRecurDays, transactionRecurUntil) {
    return $.ajax("api/add/transaction", {
        data: JSON.stringify({
            access_token,
            budget_id: Number(budgetID),
            transaction_name: transactionName,
            transaction_description: transactionDescription,
            transaction_amount: parseFloat(transactionAmount),
            transaction_recur_days: Number(transactionRecurDays),
            transaction_recur_until: transactionRecurUntil
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getBudgetTransactions(access_token, budgetID) {
    return $.ajax("api/list/transactions", {
        data: JSON.stringify({
            access_token,
            id: Number(budgetID)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getBudgetTransactionsInPeriod(access_token, budgetID, periodID) {
    return $.ajax("api/list/transactions/period", {
        data: JSON.stringify({
            access_token,
            budget_id: Number(budgetID),
            period_id: Number(periodID)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getBudgetPeriods(access_token, budgetID) {
    return $.ajax("api/list/budget_periods", {
        data: JSON.stringify({
            access_token,
            id: Number(budgetID)
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}


function getCurrentBudgetPeriod(access_token, budgetID) {
    return $.ajax("api/get/budget/current_period", {
        data: JSON.stringify({
            access_token,
            id: Number(budgetID)
        }),
        type: 'POST',
        contentType: 'application/json'
    }); 
}

function getBudgetPeriod(access_token, budgetID, periodID) {
    return $.ajax("api/get/budget/period", {
        data: JSON.stringify({
            access_token,
            budget_id: Number(budgetID),
            period_id: Number(periodID),
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function getBudgetSpent(access_token, budgetID, periodID) {
    return $.ajax("api/get/budget/spent", {
        data: JSON.stringify({
            access_token,
            budget_id: Number(budgetID),
            period_id: Number(periodID),
        }),
        type: 'POST',
        contentType: 'application/json'
    });
}

function fromSqliteDate(sdate) {

    let dparts = sdate.split("-").map(x => Number(x));

    return new Date(year = dparts[0], monthIndex = dparts[1]-1, date = dparts[2],
        hours = 0, minutes = 0, seconds = 0);
}

function fromSqliteDateTime(sdate) {
    let sparts = sdate.split(" ");

    let dparts = sparts[0].split("-").map(x => Number(x));
    let tparts = sparts[1].split(":").map(x => Number(x));

    let d = new Date(year = dparts[0], monthIndex = dparts[1]-1, date = dparts[2],
        hours = tparts[0], minutes = tparts[1], seconds = tparts[2]);

    return d;
}

function toNiceDay(day) {
    let days_of_week = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday"
    ];

    return days_of_week[day % 7];
}

function toNiceMonth(month) {
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "Novemenber",
        "December"
    ];

    return months[month % 12];
}

function localDateString(date) {
    date = new Date(date);
    return `${date.getDate()}/${date.getMonth() + 1}/${date.getFullYear()}`;
}

function niceDate(date) {
    let now = new Date();

    let seconds = (now - date)/1000;
    let minutes = seconds/60;
    let hours = minutes/60;

    let is_today = (date.getDate() == now.getDate() && hours < 24);
    let was_yesterday = (date.getDate() == (now.getDate() - 1)) && hours < 48;

    // TODO: Allow for choice of either 24 or 12 hour time

    let hour = date.getHours();
    let am_pm = (hour < 12)? 'am' : 'pm';
    
    if (hour > 12) {
        hour = hour - 12;
    }

    if (hour == 0) {
        hour = 12;
    }

    let minute = date.getMinutes();

    if (seconds < 60) {
        s = ((seconds.toFixed(0) != 1)? 's' : '');
        return `${seconds.toFixed(0)} second${s} ago`
    } else if (minutes < 60) {
        s = ((minutes.toFixed(0) != 1)? 's' : '');
        return `${minutes.toFixed(0)} minute${s} ago`
    } else if ((hours < 24) && is_today) {
        s = ((hours.toFixed(0) != 1)? 's' : '');
        return `${hours.toFixed(0)} hour${s} ago`
    } else if (was_yesterday) {
        return `${hour}:${minute} ${am_pm}, Yesterday`
    } else {
        return `${hour}:${minute} ${am_pm}, ${toNiceDay(date.getDay())}, ${date.getDate()} ${toNiceMonth(date.getMonth())} ${date.getFullYear()}`;
    }
}

function setBar(bar, percent) {
    bar.prop("style", `width: ${percent}%`)

    if(percent < 0) {
        bar.parent().addClass("progress-bar-negative");
    }

    if (percent < 15.0) {
        bar.addClass("bg-danger");
    } else if (percent < 30.0) {
        bar.addClass("bg-warning");
    } else {
        bar.addClass("bg-success");
    }
}