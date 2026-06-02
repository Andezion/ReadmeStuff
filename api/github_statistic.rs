// Use GitHub’s commit search and paginate through the results:

//     Call GET /search/commits with a query like:

//     q=author:YOUR_USERNAME

//     You can also add repo:OWNER/REPO if you want one repository.

//     Add sort=author-date or sort=committer-date if you want a time-based order.

//     Use per_page and page to keep fetching more results, since the API returns up to 100 results per page.

// Example:

// https://api.github.com/search/commits?q=author:octocat&sort=author-date&order=asc&per_page=100&page=1

